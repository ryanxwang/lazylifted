use crate::search::{
    search_engines::{InstrumentedGBFS, BFS, GBFS},
    Action, Heuristic, PreferredOperator, SearchStatistics, SuccessorGenerator, Task,
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
        preferred_operators: Option<Box<dyn PreferredOperator>>,
    ) -> (SearchResult, SearchStatistics);
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
#[clap(rename_all = "kebab-case")]
pub enum SearchEngineName {
    #[clap(help = "Breadth-first search")]
    BFS,
    #[clap(help = "Greedy best-first search")]
    GBFS,
    #[clap(
        help = "Greedy best-first search with some experimental instrumentation (requires preferred operators)"
    )]
    InstrumentedGBFS,
}

impl SearchEngineName {
    pub fn create(&self) -> Box<dyn SearchEngine> {
        match self {
            SearchEngineName::BFS => Box::new(BFS::new()),
            SearchEngineName::GBFS => Box::new(GBFS::new()),
            SearchEngineName::InstrumentedGBFS => Box::new(InstrumentedGBFS::new()),
        }
    }
}
