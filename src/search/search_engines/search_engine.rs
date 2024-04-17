use crate::search::{
    search_engines::{Bfs, Gbfs},
    Action, Heuristic, SearchStatistics, SuccessorGenerator, Task,
};

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
        heuristic: Box<dyn Heuristic>,
    ) -> (SearchResult, SearchStatistics);
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum SearchEngineName {
    #[clap(help = "Breadth-first search")]
    BFS,
    #[clap(help = "Greedy best-first search")]
    GBFS,
}

impl SearchEngineName {
    pub fn create(&self) -> Box<dyn SearchEngine> {
        match self {
            SearchEngineName::BFS => Box::new(Bfs::new()),
            SearchEngineName::GBFS => Box::new(Gbfs::new()),
        }
    }
}
