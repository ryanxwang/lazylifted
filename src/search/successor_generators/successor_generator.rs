use crate::search::{
    successor_generators::{JoinSuccessorGenerator, NaiveJoinAlgorithm},
    Action, ActionSchema, DBState, Task,
};
use clap;

pub trait SuccessorGenerator {
    fn get_applicable_actions(&self, state: &DBState, action: &ActionSchema) -> Vec<Action>;

    fn generate_successor(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        action: &Action,
    ) -> DBState;
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum SuccessorGeneratorName {
    Naive,
}

impl SuccessorGeneratorName {
    pub fn create(&self, task: &Task) -> impl SuccessorGenerator {
        match self {
            SuccessorGeneratorName::Naive => {
                JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), task)
            }
        }
    }
}
