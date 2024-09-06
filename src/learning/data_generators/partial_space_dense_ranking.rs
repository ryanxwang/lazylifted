use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, ColourDictionary, PartialActionCompilerConfig},
        models::{
            RankingPair, RankingRelation, RankingTrainingData, TrainingData, TrainingInstance,
        },
    },
    search::{successor_generators::SuccessorGeneratorName, Action, PartialAction},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialSpaceDenseRankingConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: PartialActionCompilerConfig,
    pub group_partial_actions: bool,
    pub state_sibling_weight: f64,
    pub state_predecessor_weight: f64,
    pub layer_sibling_weight: f64,
    pub layer_predecessor_weight: f64,
}

#[derive(Debug)]
pub struct PartialSpaceDenseRanking {
    config: PartialSpaceDenseRankingConfig,
}

impl PartialSpaceDenseRanking {
    pub fn new(config: &PartialSpaceDenseRankingConfig) -> Self {
        PartialSpaceDenseRanking {
            config: config.clone(),
        }
    }
}

impl DataGenerator for PartialSpaceDenseRanking {
    fn generate(
        &self,
        training_instances: &[TrainingInstance],
        colour_dictionary: &mut ColourDictionary,
    ) -> TrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();
        let mut group_ids = Vec::new();

        let mut state_predecessor_pairs_count = 0;
        let mut layer_predecessor_pairs_count = 0;
        let mut state_sibling_pairs_count = 0;
        let mut layer_sibling_pairs_count = 0;

        for instance in training_instances {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.config.successor_generator.create(task);
            let compiler = self
                .config
                .graph_compiler
                .create(task, self.config.successor_generator);

            let mut cur_state = task.initial_state.clone();

            let mut state_predecessor_index: Option<usize> = None;
            let mut layer_predecessor_index: Option<usize> = None;
            for chosen_action in plan.steps() {
                let applicable_actions: Vec<Action> = task
                    .action_schemas()
                    .iter()
                    .flat_map(|schema| -> Vec<Action> {
                        successor_generator.get_applicable_actions(&cur_state, schema)
                    })
                    .collect();

                let partial_actions: Vec<PartialAction> = applicable_actions
                    .iter()
                    .flat_map(|action| {
                        (0..=action.instantiation.len())
                            .map(|depth| PartialAction::from_action(action, depth))
                            .collect::<Vec<PartialAction>>()
                    })
                    .unique()
                    .collect();

                for partial_depth in 0..=(chosen_action.instantiation.len()) {
                    let chosen_partial = PartialAction::from_action(chosen_action, partial_depth);

                    let graph =
                        compiler.compile(&cur_state, &chosen_partial, Some(colour_dictionary));
                    let cur_index = graphs.len();
                    graphs.push(graph.clone());
                    group_ids.push(chosen_partial.group_id());

                    // First rank this partial action better than its layer predecessor
                    if let Some(layer_predecessor_index) = &layer_predecessor_index {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: *layer_predecessor_index,
                            relation: RankingRelation::Better,
                            importance: self.config.layer_predecessor_weight,
                        });
                        layer_predecessor_pairs_count += 1;
                    }
                    layer_predecessor_index = Some(cur_index);

                    // Also rank it better than its state predecessor, note that
                    // we update the state predecessor only for final partials,
                    // hence the distinction between layer and state
                    // predecessors
                    if let Some(state_predecessor_index) = &state_predecessor_index {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: *state_predecessor_index,
                            relation: RankingRelation::Better,
                            importance: self.config.state_predecessor_weight,
                        });
                        state_predecessor_pairs_count += 1;
                    }

                    // Add layer siblings for all partials
                    for partial in &partial_actions {
                        // A layer sibling is one that is not final in the same
                        // layer
                        if partial.depth()
                            != task.action_schemas()[partial.schema_index()]
                                .parameters()
                                .len()
                            && partial != &chosen_partial
                            && partial.depth() == chosen_partial.depth()
                        {
                            pairs.push(RankingPair {
                                i: cur_index,
                                j: graphs.len(),
                                relation: RankingRelation::BetterOrEqual,
                                importance: self.config.layer_sibling_weight,
                            });
                            graphs.push(compiler.compile(
                                &cur_state,
                                partial,
                                Some(colour_dictionary),
                            ));
                            group_ids.push(partial.group_id());
                            layer_sibling_pairs_count += 1;
                        }
                    }

                    // Add state siblings for final partials only
                    let is_final_partial = partial_depth == chosen_action.instantiation.len();
                    if !is_final_partial {
                        continue;
                    }
                    state_predecessor_index = Some(cur_index);
                    for partial in &partial_actions {
                        // A state sibling is one that is also final
                        if partial.depth()
                            == task.action_schemas()[partial.schema_index()]
                                .parameters()
                                .len()
                            && partial != &chosen_partial
                        {
                            pairs.push(RankingPair {
                                i: cur_index,
                                j: graphs.len(),
                                relation: RankingRelation::BetterOrEqual,
                                importance: self.config.state_sibling_weight,
                            });
                            graphs.push(compiler.compile(
                                &cur_state,
                                partial,
                                Some(colour_dictionary),
                            ));
                            group_ids.push(partial.group_id());
                            state_sibling_pairs_count += 1;
                        }
                    }
                }

                cur_state = successor_generator.generate_successor(
                    &cur_state,
                    &task.action_schemas()[chosen_action.index],
                    chosen_action,
                );
            }
        }

        info!(
            state_predecessor_pairs_count = state_predecessor_pairs_count,
            layer_predecessor_pairs_count = layer_predecessor_pairs_count,
            state_sibling_pairs_count = state_sibling_pairs_count,
            layer_sibling_pairs_count = layer_sibling_pairs_count,
        );

        TrainingData::Ranking(RankingTrainingData {
            features: graphs,
            pairs,
            group_ids: if self.config.group_partial_actions {
                Some(group_ids)
            } else {
                None
            },
        })
    }
}
