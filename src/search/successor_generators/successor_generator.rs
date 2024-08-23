use crate::search::{
    successor_generators::{FullReducer, JoinSuccessorGenerator, NaiveJoinAlgorithm},
    Action, ActionSchema, DBState, PartialAction, Task,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait SuccessorGenerator: Debug {
    fn get_applicable_actions(&self, state: &DBState, action_schema: &ActionSchema) -> Vec<Action>;

    // Get applicable actions that satisfy the fixed parameters from the partial
    // action
    fn get_applicable_actions_from_partial(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        partial_action: &PartialAction,
    ) -> Vec<Action>;

    fn generate_successor(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        action: &Action,
    ) -> DBState;
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Deserialize, Serialize)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum SuccessorGeneratorName {
    Naive,
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
