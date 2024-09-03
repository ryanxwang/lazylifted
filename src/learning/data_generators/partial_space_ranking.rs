use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, ColourDictionary, PartialActionCompilerName},
        models::{
            RankingPair, RankingRelation, RankingTrainingData, TrainingData, TrainingInstance,
        },
    },
    search::{successor_generators::SuccessorGeneratorName, Action, PartialAction},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialSpaceRankingConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: PartialActionCompilerName,
    pub group_partial_actions: bool,
}

#[derive(Debug)]
pub struct PartialSpaceRanking {
    config: PartialSpaceRankingConfig,
}

impl PartialSpaceRanking {
    pub fn new(config: &PartialSpaceRankingConfig) -> Self {
        PartialSpaceRanking {
            config: config.clone(),
        }
    }
}

impl DataGenerator for PartialSpaceRanking {
    fn generate(
        &self,
        training_instances: &[TrainingInstance],
        colour_dictionary: &mut ColourDictionary,
    ) -> TrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();
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

            let mut predecessor_index: Option<usize> = None;
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

                    // First rank this partial action better than its predecessor
                    if let Some(predecessor_index) = &predecessor_index {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: *predecessor_index,
                            relation: RankingRelation::Better,
                            importance: 1.,
                        });
                    }
                    predecessor_index = Some(cur_index);

                    // Only compare to siblings if both are final
                    let is_final_partial = partial_depth == chosen_action.instantiation.len();
                    if !is_final_partial {
                        continue;
                    }
                    for partial in &partial_actions {
                        if partial.depth()
                            == task.action_schemas()[partial.schema_index()]
                                .parameters()
                                .len()
                        {
                            pairs.push(RankingPair {
                                i: cur_index,
                                j: graphs.len(),
                                relation: RankingRelation::BetterOrEqual,
                                importance: 1.,
                            });
                            graphs.push(compiler.compile(
                                &cur_state,
                                partial,
                                Some(colour_dictionary),
                            ));
                            group_ids.push(partial.group_id());
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
