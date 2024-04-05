use crate::search::heuristics::goal_counting::GoalCounting;
use crate::search::{DBState, Task};
use clap;

pub trait Heuristic {
    /// Evaluate the given state with respect to the given task.
    fn evaluate(&mut self, state: &DBState, task: &Task) -> f64;

    /// Evaluate a batch of states with respect to the given task. The default
    /// implementation simply calls `evaluate` for each state sequentially. This
    /// method should be overridden if a more efficient implementation is
    /// possible.
    fn evaluate_batch(&mut self, states: &[DBState], task: &Task) -> Vec<f64> {
        states
            .iter()
            .map(|state| self.evaluate(state, task))
            .collect()
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum HeuristicName {
    GoalCounting,
}

impl HeuristicName {
    pub fn create(&self) -> impl Heuristic {
        match self {
            HeuristicName::GoalCounting => GoalCounting::new(),
        }
    }
}
