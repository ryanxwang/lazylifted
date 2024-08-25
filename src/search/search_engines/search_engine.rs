use crate::search::{
    search_engines::{Bfs, Gbfs, TerminationCondition},
    Plan, SearchProblem, Transition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResult {
    /// The search was successful
    Success(Plan),
    /// The search was provably unsolvable
    ProvablyUnsolvable,
    /// The search was unsolvable, but the search engine is also incomplete
    IncompleteUnsolvable,
    /// The search engine ran out of memory
    MemoryLimitExceeded,
    /// The search engine ran out of time
    TimeLimitExceeded,
}

pub trait SearchEngine<S, T> {
    fn search(
        &self,
        problem: Box<dyn SearchProblem<S, T>>,
        termination_condition: TerminationCondition,
    ) -> SearchResult;
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
    pub fn search<S, T: Transition>(
        &self,
        problem: Box<dyn SearchProblem<S, T>>,
        termination_condition: TerminationCondition,
    ) -> SearchResult {
        let engine: Box<dyn SearchEngine<S, T>> = match self {
            SearchEngineName::BFS => Box::new(Bfs::new()),
            SearchEngineName::GBFS => Box::new(Gbfs::new()),
        };
        engine.search(problem, termination_condition)
    }
}
