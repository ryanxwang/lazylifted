use std::fmt::Debug;

use crate::learning::graphs::{AoagCompiler, ColourDictionary, IlgCompiler, RslgCompiler};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, PartialAction, Task};
use petgraph::{graph::Graph, Undirected};
use serde::{Deserialize, Serialize};

pub type CGraph = Graph<usize, usize, Undirected, u32>;
pub type NodeID = petgraph::graph::NodeIndex<u32>;

pub trait Compiler<T>: Debug {
    #[allow(dead_code)]
    fn compile(&self, arg: &T, colour_dictionary: Option<&mut ColourDictionary>) -> CGraph;
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum StateCompilerName {
    Ilg,
}

impl StateCompilerName {
    pub fn create(&self, task: &Task) -> Box<dyn Compiler<DBState>> {
        match self {
            StateCompilerName::Ilg => Box::new(IlgCompiler::new(task)),
        }
    }
}

pub trait PartialActionCompiler: Debug {
    /// Compile the (state, partial) pair into a graph.
    fn compile(
        &self,
        state: &DBState,
        partial: &PartialAction,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph;
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum PartialActionCompilerName {
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
