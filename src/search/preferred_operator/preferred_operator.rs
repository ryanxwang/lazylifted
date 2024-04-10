use crate::search::{preferred_operator::wl_aslg::WLASLGPrefOp, Action, DBState, Task};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait PreferredOperator {
    /// Rethrn a vector of booleans with the same length as the number of
    /// actions in the task. Each boolean indicates whether the corresponding
    /// action is a preferred operator.
    fn preferred_operators(
        &mut self,
        state: &DBState,
        task: &Task,
        actions: &[Action],
    ) -> Vec<bool>;
}

#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize)]
pub enum PreferredOperatorName {
    #[clap(
        name = "wl-aslg",
        help = "Using the WL-ALSG ranker to compute preferred operators"
    )]
    WLASLG,
}

impl PreferredOperatorName {
    pub fn create(&self, saved_model: &PathBuf) -> Box<dyn PreferredOperator> {
        match self {
            PreferredOperatorName::WLASLG => Box::new(WLASLGPrefOp::load(saved_model)),
        }
    }
}
