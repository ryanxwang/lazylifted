use std::fmt::Debug;

use crate::learning::graphs::{
    AoagCompiler, AoagConfig, ColourDictionary, IlgCompiler, IlgConfig, RslgCompiler, RslgConfig,
};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateCompilerConfig {
    Ilg(IlgConfig),
    Aoag(AoagConfig),
    Rslg(RslgConfig),
}

impl StateCompilerConfig {
    pub fn create(
        &self,
        task: &Task,
        successor_generator_name: SuccessorGeneratorName,
    ) -> Box<dyn Compiler<DBState>> {
        match self {
            StateCompilerConfig::Ilg(config) => Box::new(IlgCompiler::new(task, config)),
            StateCompilerConfig::Aoag(config) => {
                Box::new(AoagCompiler::new(task, successor_generator_name, config))
            }
            StateCompilerConfig::Rslg(config) => {
                Box::new(RslgCompiler::new(task, successor_generator_name, config))
            }
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

// TODO-soon: might be a good idea to merge the two graph config types, not sure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PartialActionCompilerConfig {
    Ilg(IlgConfig),
    Rslg(RslgConfig),
    Aoag(AoagConfig),
}

impl PartialActionCompilerConfig {
    pub fn create(
        &self,
        task: &Task,
        successor_generator_name: SuccessorGeneratorName,
    ) -> Box<dyn PartialActionCompiler> {
        match self {
            PartialActionCompilerConfig::Ilg(config) => Box::new(IlgCompiler::new(task, config)),
            PartialActionCompilerConfig::Rslg(config) => {
                Box::new(RslgCompiler::new(task, successor_generator_name, config))
            }
            PartialActionCompilerConfig::Aoag(config) => {
                Box::new(AoagCompiler::new(task, successor_generator_name, config))
            }
        }
    }

    pub fn to_state_space_compiler_config(&self) -> StateCompilerConfig {
        match self {
            PartialActionCompilerConfig::Ilg(config) => StateCompilerConfig::Ilg(config.clone()),
            PartialActionCompilerConfig::Rslg(config) => StateCompilerConfig::Rslg(config.clone()),
            PartialActionCompilerConfig::Aoag(config) => StateCompilerConfig::Aoag(config.clone()),
        }
    }
}
