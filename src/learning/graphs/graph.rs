use std::fmt::Debug;

use crate::learning::graphs::{IlgCompiler, PalgCompiler, RslgCompiler, SclgCompiler};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, PartialAction, Task};
use petgraph::{graph::Graph, Undirected};
use serde::{Deserialize, Serialize};

pub type CGraph = Graph<i32, i32, Undirected, u32>;
pub type NodeID = petgraph::graph::NodeIndex<u32>;

pub trait Compiler<T>: Debug {
    #[allow(dead_code)]
    fn compile(&self, arg: &T) -> CGraph;
}

pub trait PartialActionCompiler: Debug {
    /// Compile the (state, partial) pair into a graph.
    fn compile(&self, state: &DBState, partial: &PartialAction) -> CGraph;

    // /// Get the concentration of the partial action in the state. This is a
    // /// number between 0 and 1 that represents how much certainty the compiled
    // /// graph has on the effect of the partial action. For example, if the
    // /// partial action is actually a full action, the concentration should be 1,
    // /// and if the partial action has a wide range of possible effects (for
    // /// example, (stack b1 ?underob)), the concentration should be close to 0.
    // fn get_partial_concentration(&self, state: &DBState, partial: &PartialAction) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum PartialActionCompilerName {
    Sclg,
    Palg,
    Ilg,
    Rslg,
}

impl PartialActionCompilerName {
    pub fn create(
        &self,
        task: &Task,
        successor_generator_name: SuccessorGeneratorName,
    ) -> Box<dyn PartialActionCompiler> {
        match self {
            PartialActionCompilerName::Sclg => {
                Box::new(SclgCompiler::new(task, successor_generator_name))
            }
            PartialActionCompilerName::Palg => Box::new(PalgCompiler::new(task)),
            PartialActionCompilerName::Ilg => Box::new(IlgCompiler::new(task)),
            PartialActionCompilerName::Rslg => {
                Box::new(RslgCompiler::new(task, successor_generator_name))
            }
        }
    }
}
