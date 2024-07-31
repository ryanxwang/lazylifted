use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, IlgCompiler},
        models::{
            RankingPair, RankingRelation, RankingTrainingData, TrainingData, TrainingInstance,
        },
    },
    search::{successor_generators::SuccessorGeneratorName, Action},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateSpaceIlgRankingConfig {
    pub successor_generator: SuccessorGeneratorName,
}

#[derive(Debug)]
pub struct StateSpaceIlgRanking {
    config: StateSpaceIlgRankingConfig,
}

impl StateSpaceIlgRanking {
    pub fn new(config: &StateSpaceIlgRankingConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl DataGenerator for StateSpaceIlgRanking {
    fn generate(&self, training_instances: &[TrainingInstance]) -> TrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();

        for instance in training_instances {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.config.successor_generator.create(task);
            let compiler = IlgCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            let mut predecessor_graph: Option<CGraph> = None;
            let mut sibling_graphs: Option<Vec<CGraph>> = None;
            for chosen_action in plan.steps() {
                let cur_graph = compiler.compile(&cur_state);
                let cur_index = graphs.len();
                graphs.push(cur_graph.clone());

                // First rank this state better than its predecessors
                if let Some(predecessor_graph) = &predecessor_graph {
                    pairs.push(RankingPair {
                        i: cur_index,
                        j: graphs.len(),
                        relation: RankingRelation::Better,
                    });
                    graphs.push(predecessor_graph.clone());
                }

                // Then rank it better than or equal to its siblings
                if let Some(sibling_graphs) = &sibling_graphs {
                    for sibling_graph in sibling_graphs {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: graphs.len(),
                            relation: RankingRelation::BetterOrEqual,
                        });
                        graphs.push(sibling_graph.clone());
                    }
                }

                // Update the structs
                sibling_graphs = Some(vec![]);
                let applicable_actions: Vec<Action> = task
                    .action_schemas()
                    .iter()
                    .flat_map(|schema| {
                        successor_generator.get_applicable_actions(&cur_state, schema)
                    })
                    .collect();
                for action in applicable_actions {
                    if action == *chosen_action {
                        continue;
                    }

                    let action_schema = &task.action_schemas()[action.index];
                    let next_state =
                        successor_generator.generate_successor(&cur_state, action_schema, &action);
                    let next_graph = compiler.compile(&next_state);
                    sibling_graphs.as_mut().unwrap().push(next_graph);
                }

                predecessor_graph = Some(cur_graph);

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
            group_ids: None,
        })
    }
}
