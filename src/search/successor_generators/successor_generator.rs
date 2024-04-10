use crate::search::{
    successor_generators::{FullReducer, JoinSuccessorGenerator, NaiveJoinAlgorithm},
    Action, ActionSchema, DBState, Task,
};
use serde::{Deserialize, Serialize};

pub trait SuccessorGenerator {
    fn get_applicable_actions(&self, state: &DBState, action: &ActionSchema) -> Vec<Action>;

    fn generate_successor(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        action: &Action,
    ) -> DBState;
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
pub enum SuccessorGeneratorName {
    #[serde(alias = "naive")]
    Naive,
    #[serde(alias = "full-reducer")]
    FullReducer,
}

impl SuccessorGeneratorName {
    pub fn create(&self, task: &Task) -> Box<dyn SuccessorGenerator> {
        match self {
            SuccessorGeneratorName::Naive => {
                Box::new(JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), task))
            }
            SuccessorGeneratorName::FullReducer => {
                Box::new(JoinSuccessorGenerator::new(FullReducer::new(task), task))
            }
        }
    }
}
