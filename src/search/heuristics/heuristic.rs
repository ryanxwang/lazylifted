use crate::search::heuristics::{
    FfHeuristic, GoalCounting, HaddHeuristic, HmaxHeuristic, Requirement, WlPartialHeuristic,
    WlStateHeuristic, ZeroHeuristic,
};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, PartialAction, Task};
use ordered_float::OrderedFloat;
use std::collections::HashSet;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;

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
    #[clap(
        name = "wl",
        help = "The WL heuristic, requires a model file with a trained state space model."
    )]
    Wl,
    #[clap(help = "The goal counting heuristic.")]
    GoalCounting,
    #[clap(
        name = "hmax",
        help = "The h_max heuristic. Will require removing negative \
        preconditions if they exist, please use the command line option for it."
    )]
    Hmax,
    #[clap(
        name = "hadd",
        help = "The h_add heuristic. Will require removing negative \
        preconditions if they exist, please use the command line option for it."
    )]
    Hadd,
    #[clap(
        name = "ff",
        help = "The FF heuristic. Will require removing negative \
        preconditions if they exist, please use the command line option for it."
    )]
    Ff,
    #[clap(name = "zero", help = "The zero heuristic.")]
    ZeroHeuristic,
}

impl StateHeuristicNames {
    pub fn create(
        &self,
        task: Rc<Task>,
        successor_generator_name: SuccessorGeneratorName,
        saved_model: Option<&Path>,
    ) -> Box<dyn Heuristic<DBState>> {
        match self {
            StateHeuristicNames::GoalCounting => {
                Box::new(GoalCounting::new(task.clone(), successor_generator_name))
            }
            StateHeuristicNames::Hmax => Box::new(HmaxHeuristic::new(task.clone())),
            StateHeuristicNames::Hadd => Box::new(HaddHeuristic::new(task.clone())),
            StateHeuristicNames::Ff => Box::new(FfHeuristic::new(task.clone(), false)),
            StateHeuristicNames::Wl => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL heuristic");
                Box::new(WlStateHeuristic::load(saved_model))
            }
            StateHeuristicNames::ZeroHeuristic => Box::new(ZeroHeuristic::new()),
        }
    }

    pub fn requirements(&self) -> HashSet<Requirement> {
        match self {
            StateHeuristicNames::GoalCounting
            | StateHeuristicNames::ZeroHeuristic
            | StateHeuristicNames::Wl => HashSet::new(),
            StateHeuristicNames::Ff | StateHeuristicNames::Hmax | StateHeuristicNames::Hadd => {
                HashSet::from([Requirement::NoNegativePreconditions])
            }
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum PartialActionHeuristicNames {
    #[clap(
        name = "wl",
        help = "Run an WL based heuristic, requires a model file with a trained partial action model."
    )]
    Wl,
    #[clap(name = "zero", help = "The zero heuristic.")]
    ZeroHeuristic,
    #[clap(help = "The goal counting heuristic.")]
    GoalCounting,
}

impl PartialActionHeuristicNames {
    pub fn create(
        &self,
        task: Rc<Task>,
        successor_generator_name: SuccessorGeneratorName,
        saved_model: Option<&Path>,
    ) -> Box<dyn Heuristic<(DBState, PartialAction)>> {
        match self {
            PartialActionHeuristicNames::Wl => {
                let saved_model = saved_model
                    .as_ref()
                    .expect("No saved model provided for WL heuristic");
                Box::new(WlPartialHeuristic::load(saved_model))
            }
            PartialActionHeuristicNames::ZeroHeuristic => Box::new(ZeroHeuristic::new()),
            PartialActionHeuristicNames::GoalCounting => {
                Box::new(GoalCounting::new(task.clone(), successor_generator_name))
            }
        }
    }

    pub fn requirements(&self) -> HashSet<Requirement> {
        HashSet::new()
    }
}
