use std::fmt::Debug;

use crate::learning::graphs::{IlgCompiler, PalgCompiler, SclgCompiler};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, PartialAction, Task};
use petgraph::{graph::Graph, Undirected};
use serde::{Deserialize, Serialize};

pub type CGraph = Graph<i32, i32, Undirected, u32>;
pub type NodeID = petgraph::graph::NodeIndex<u32>;

pub trait Compiler<T>: Debug {
    fn compile(&self, arg: &T) -> CGraph;
}

pub trait Compiler2<T, U>: Debug {
    fn compile(&self, arg1: &T, arg2: &U) -> CGraph;
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum PartialActionCompilerName {
    Sclg,
    Palg,
    Ilg,
}

impl PartialActionCompilerName {
    pub fn create(
        &self,
        task: &Task,
        successcor_generator_name: SuccessorGeneratorName,
    ) -> Box<dyn Compiler2<DBState, PartialAction>> {
        match self {
            PartialActionCompilerName::Sclg => {
                Box::new(SclgCompiler::new(task, successcor_generator_name))
            }
            PartialActionCompilerName::Palg => Box::new(PalgCompiler::new(task)),
            PartialActionCompilerName::Ilg => Box::new(IlgCompiler::new(task)),
        }
    }
}
