use crate::search::heuristics::goal_counting::GoalCounting;
use crate::search::heuristics::wl_ilg::WLILGHeuristic;
use crate::search::{DBState, Task};
use ordered_float::OrderedFloat;
use std::path::PathBuf;

pub type HeuristicValue = OrderedFloat<f64>;

pub trait Heuristic {
    /// Evaluate the given state with respect to the given task.
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue;

    /// Evaluate a batch of states with respect to the given task. The default
    /// implementation simply calls `evaluate` for each state sequentially. This
    /// method should be overridden if a more efficient implementation is
    /// possible.
    fn evaluate_batch(&mut self, states: &[DBState], task: &Task) -> Vec<HeuristicValue> {
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
    #[clap(name = "wl-ilg", help = "The WL-ILG heuristic, requires a model file.")]
    WLILG,
}

impl HeuristicName {
    pub fn create(&self, saved_model: &Option<PathBuf>) -> Box<dyn Heuristic> {
        match self {
            HeuristicName::GoalCounting => Box::new(GoalCounting::new()),
            HeuristicName::WLILG => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL-ILG heuristic");
                Box::new(WLILGHeuristic::load(saved_model))
            }
        }
    }
}
