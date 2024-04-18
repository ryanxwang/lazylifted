//! Breadth first search

use crate::search::{
    search_engines::{SearchEngine, SearchResult},
    Plan, SearchProblem, StateId, Transition,
};
use std::collections::VecDeque;

pub struct Bfs {}

impl Bfs {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, T> SearchEngine<S, T> for Bfs
where
    T: Transition,
{
    fn search(&self, mut problem: Box<dyn SearchProblem<S, T>>) -> SearchResult {
        if problem.is_goal(problem.initial_state().get_state_id()) {
            return SearchResult::Success(Plan::empty());
        }

        let mut queue = VecDeque::new();
        queue.push_back(problem.initial_state().get_state_id());

        while !queue.is_empty() {
            let state_id = queue.pop_front().unwrap();

            let successors_ids: Vec<StateId> = problem
                .expand(state_id)
                .iter()
                .map(|successor| successor.get_state_id())
                .collect();
            for id in successors_ids {
                if problem.is_goal(id) {
                    return SearchResult::Success(problem.extract_plan(id));
                }
                queue.push_back(id);
            }
        }

        SearchResult::ProvablyUnsolvable
    }
}
