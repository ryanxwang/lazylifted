use crate::search::{search_engines::BFS, Heuristic, SuccessorGenerator, Task, Verbosity};
use clap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchResult {
    /// The search was successful
    Success,
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
        generator: &impl SuccessorGenerator,
        heuristic: &impl Heuristic,
    ) -> SearchResult;
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum SearchEngineName {
    BFS,
}

impl SearchEngineName {
    pub fn create(&self, verbosity: Verbosity) -> impl SearchEngine {
        match self {
            SearchEngineName::BFS => BFS::new(verbosity),
        }
    }
}
