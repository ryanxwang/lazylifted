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
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialSpaceDenseRankingConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: PartialActionCompilerConfig,
    pub group_partial_actions: bool,
    pub total_state_sibling_ratio: f64,
    pub total_state_predecessor_ratio: f64,
    pub total_layer_sibling_ratio: f64,
    pub total_layer_predecessor_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum PartialSpaceDenseRankingType {
    StatePredecessor,
    LayerPredecessor,
    StateSibling,
    LayerSibling,
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
        let mut pair_types = Vec::new();
        let mut group_ids = Vec::new();

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
                            importance: 0.0,
                        });
                        pair_types.push(PartialSpaceDenseRankingType::LayerPredecessor);
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
                            importance: 0.0,
                        });
                        pair_types.push(PartialSpaceDenseRankingType::StatePredecessor);
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
                                importance: 0.0,
                            });
                            graphs.push(compiler.compile(
                                &cur_state,
                                partial,
                                Some(colour_dictionary),
                            ));
                            group_ids.push(partial.group_id());
                            pair_types.push(PartialSpaceDenseRankingType::LayerSibling);
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
                                importance: 0.0,
                            });
                            graphs.push(compiler.compile(
                                &cur_state,
                                partial,
                                Some(colour_dictionary),
                            ));
                            group_ids.push(partial.group_id());
                            pair_types.push(PartialSpaceDenseRankingType::StateSibling);
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

        let max_count = PartialSpaceDenseRankingType::iter()
            .map(|t| pair_types.iter().filter(|&&t2| t2 == t).count())
            .max()
            .unwrap() as f64;

        let mut distribute_weights = |t: PartialSpaceDenseRankingType, weight: f64| {
            let count = pair_types.iter().filter(|&&t2| t2 == t).count();
            let individual_weight = weight / count as f64;

            for (pair, pair_type) in pairs.iter_mut().zip(pair_types.iter()) {
                if *pair_type == t {
                    pair.importance = individual_weight;
                }
            }
            info!(pair_type = ?t, individual_weight = individual_weight, count = count);
        };
        distribute_weights(
            PartialSpaceDenseRankingType::StatePredecessor,
            self.config.total_state_predecessor_ratio * max_count,
        );
        distribute_weights(
            PartialSpaceDenseRankingType::LayerPredecessor,
            self.config.total_layer_predecessor_ratio * max_count,
        );
        distribute_weights(
            PartialSpaceDenseRankingType::StateSibling,
            self.config.total_state_sibling_ratio * max_count,
        );
        distribute_weights(
            PartialSpaceDenseRankingType::LayerSibling,
            self.config.total_layer_sibling_ratio * max_count,
        );

        TrainingData::Ranking(RankingTrainingData {
            features: graphs,
            pairs,
        })
    }
}
