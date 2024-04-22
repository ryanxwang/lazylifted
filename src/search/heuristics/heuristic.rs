use crate::search::heuristics::goal_counting::GoalCounting;
use crate::search::heuristics::wl_ilg::WlIlgHeuristic;
use crate::search::heuristics::wl_palg::WlPalgHeuristic;
use crate::search::heuristics::zero_heuristic::ZeroHeuristic;
use crate::search::{DBState, PartialAction, Task};
use ordered_float::OrderedFloat;
use std::fmt::Debug;
use std::path::Path;

pub type HeuristicValue = OrderedFloat<f64>;

pub trait Heuristic<T>: Debug {
    /// Evaluate the given state with respect to the given task.
    fn evaluate(&mut self, state: &T, task: &Task) -> HeuristicValue;

    /// Evaluate a batch of states with respect to the given task. The default
    /// implementation simply calls `evaluate` for each state sequentially. This
    /// method should be overridden if a more efficient implementation is
    /// possible.
    fn evaluate_batch(&mut self, states: &[T], task: &Task) -> Vec<HeuristicValue> {
        states
            .iter()
            .map(|state| self.evaluate(state, task))
            .collect()
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum StateHeuristicNames {
    #[clap(name = "wl-ilg", help = "The WL-ILG heuristic, requires a model file.")]
    WlIlg,
    #[clap(help = "The goal counting heuristic.")]
    GoalCounting,
    #[clap(name = "zero", help = "The zero heuristic.")]
    ZeroHeuristic,
}

impl StateHeuristicNames {
    pub fn create(
        &self,
        model_config: Option<&Path>,
        saved_model: Option<&Path>,
    ) -> Box<dyn Heuristic<DBState>> {
        match self {
            StateHeuristicNames::GoalCounting => Box::new(GoalCounting::new()),
            StateHeuristicNames::WlIlg => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL-ILG heuristic");
                let model_config = model_config
                    .as_ref()
                    .expect("No model config provided for WL-ILG heuristic");
                Box::new(WlIlgHeuristic::load(model_config, saved_model))
            }
            StateHeuristicNames::ZeroHeuristic => Box::new(ZeroHeuristic::new()),
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum PartialActionHeuristicNames {
    #[clap(
        name = "wl-palg",
        help = "The WL-PALG heuristic, requires a model file."
    )]
    WlPalg,
    #[clap(name = "zero", help = "The zero heuristic.")]
    ZeroHeuristic,
}

impl PartialActionHeuristicNames {
    pub fn create(
        &self,
        config_path: Option<&Path>,
        saved_model: Option<&Path>,
    ) -> Box<dyn Heuristic<(DBState, PartialAction)>> {
        match self {
            PartialActionHeuristicNames::WlPalg => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL-PALG heuristic");
                let config_path = config_path
                    .as_ref()
                    .expect("No model config provided for WL-PALG heuristic");
                Box::new(WlPalgHeuristic::load(config_path, saved_model))
            }
            PartialActionHeuristicNames::ZeroHeuristic => Box::new(ZeroHeuristic::new()),
        }
    }
}
