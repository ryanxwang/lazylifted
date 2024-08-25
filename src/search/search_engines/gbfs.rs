//! This module implements the greedy best-first search algorithm.

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::search::{
    search_engines::{SearchEngine, SearchResult, TerminationCondition},
    HeuristicValue, NodeId, SearchProblem, Transition,
};
use std::cmp::Reverse;

/// Greedy best-first search
pub struct Gbfs {}

impl Gbfs {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S, T> SearchEngine<S, T> for Gbfs
where
    T: Transition,
{
    fn search(
        &self,
        mut problem: Box<dyn SearchProblem<S, T>>,
        mut termination_condition: TerminationCondition,
    ) -> SearchResult {
        let mut priority_queue = PriorityQueue::new();
        priority_queue.push(
            problem.initial_state().get_node_id(),
            Reverse(OrderedFloat(0.)),
        );

        while !priority_queue.is_empty() {
            termination_condition.log_if_needed();
            if let Some(result) = termination_condition.should_terminate() {
                termination_condition.finalise();
                return result;
            }
            let sid = priority_queue.pop().unwrap().0;

            if problem.is_goal(sid) {
                termination_condition.finalise();
                return SearchResult::Success(problem.extract_plan(sid));
            }

            let successors_ids_and_h_values: Vec<(NodeId, HeuristicValue)> = problem
                .expand(sid)
                .iter()
                .map(|successor| (successor.get_node_id(), successor.get_h()))
                .collect();

            for (id, h_value) in successors_ids_and_h_values {
                priority_queue.push(id, Reverse(h_value));
            }
        }

        termination_condition.finalise();
        SearchResult::ProvablyUnsolvable
    }
}
