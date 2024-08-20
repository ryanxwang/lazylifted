use std::fmt::Debug;

use crate::learning::graphs::{
    AoagCompiler, IlgCompiler, PalgCompiler, RslgCompiler, SclgCompiler,
};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, PartialAction, Task};
use petgraph::{graph::Graph, Undirected};
use serde::{Deserialize, Serialize};

pub type CGraph = Graph<usize, usize, Undirected, u32>;
pub type NodeID = petgraph::graph::NodeIndex<u32>;

pub trait Compiler<T>: Debug {
    #[allow(dead_code)]
    fn compile(&self, arg: &T) -> CGraph;
}

pub trait PartialActionCompiler: Debug {
    /// Compile the (state, partial) pair into a graph.
    fn compile(&self, state: &DBState, partial: &PartialAction) -> CGraph;
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum PartialActionCompilerName {
    Sclg,
    Palg,
    Ilg,
    Rslg,
    Aoag,
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
            PartialActionCompilerName::Aoag => {
                Box::new(AoagCompiler::new(task, successor_generator_name))
            }
        }
    }
}
