use crate::search::{
    search_engines::{SearchStatistics, BFS},
    Action, Heuristic, SuccessorGenerator, Task,
};
use clap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResult {
    /// The search was successful
    Success(Vec<Action>),
    /// The search was provably unsolvable
    ProvablyUnsolvable,
    /// The search was unsolvable, but the search engine is also incomplete
    IncompleteUnsolvable,
    /// The search engine ran out of memory
    OutOfMemory,
    /// The search engine ran out of time
    TimeLimitExceeded,
}

pub trait SearchEngine {
    fn search(
        &mut self,
        task: &Task,
        generator: Box<dyn SuccessorGenerator>,
        heuristic: &impl Heuristic,
    ) -> (SearchResult, SearchStatistics);
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum SearchEngineName {
    BFS,
}

impl SearchEngineName {
    pub fn create(&self) -> impl SearchEngine {
        match self {
            SearchEngineName::BFS => BFS::new(),
        }
    }
}
