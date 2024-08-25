//! Breadth first search

use crate::search::{
    search_engines::{SearchEngine, SearchResult, TerminationCondition},
    NodeId, Plan, SearchProblem, Transition,
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
    fn search(
        &self,
        mut problem: Box<dyn SearchProblem<S, T>>,
        mut termination_condition: TerminationCondition,
    ) -> SearchResult {
        if problem.is_goal(problem.initial_state().get_node_id()) {
            termination_condition.finalise();
            return SearchResult::Success(Plan::empty());
        }

        let mut queue = VecDeque::new();
        queue.push_back(problem.initial_state().get_node_id());

        while !queue.is_empty() {
            termination_condition.log_if_needed();
            if let Some(result) = termination_condition.should_terminate() {
                termination_condition.finalise();
                return result;
            }
            let node_id = queue.pop_front().unwrap();

            let successors_ids: Vec<NodeId> = problem
                .expand(node_id)
                .iter()
                .map(|successor| successor.get_node_id())
                .collect();
            for id in successors_ids {
                if problem.is_goal(id) {
                    termination_condition.finalise();
                    return SearchResult::Success(problem.extract_plan(id));
                }
                queue.push_back(id);
            }
        }

        termination_condition.finalise();
        SearchResult::ProvablyUnsolvable
    }
}
