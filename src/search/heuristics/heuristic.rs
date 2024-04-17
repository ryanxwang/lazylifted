use crate::search::heuristics::goal_counting::GoalCounting;
use crate::search::heuristics::wl_ilg::WlIlgHeuristic;
use crate::search::{DBState, Task};
use ordered_float::OrderedFloat;
use std::fmt::Debug;
use std::path::PathBuf;

pub type HeuristicValue = OrderedFloat<f64>;

pub trait Heuristic: Debug {
    type Target;

    /// Evaluate the given state with respect to the given task.
    fn evaluate(&mut self, state: &Self::Target, task: &Task) -> HeuristicValue;

    /// Evaluate a batch of states with respect to the given task. The default
    /// implementation simply calls `evaluate` for each state sequentially. This
    /// method should be overridden if a more efficient implementation is
    /// possible.
    fn evaluate_batch(&mut self, states: &[Self::Target], task: &Task) -> Vec<HeuristicValue> {
        states
            .iter()
            .map(|state| self.evaluate(state, task))
            .collect()
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum StateHeuristicNames {
    #[clap(help = "The goal counting heuristic.")]
    GoalCounting,
    #[clap(name = "wl-ilg", help = "The WL-ILG heuristic, requires a model file.")]
    WlIlg,
}

impl StateHeuristicNames {
    pub fn create(&self, saved_model: &Option<PathBuf>) -> Box<dyn Heuristic<Target = DBState>> {
        match self {
            StateHeuristicNames::GoalCounting => Box::new(GoalCounting::new()),
            StateHeuristicNames::WlIlg => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL-ILG heuristic");
                Box::new(WlIlgHeuristic::load(saved_model))
            }
        }
    }
}
