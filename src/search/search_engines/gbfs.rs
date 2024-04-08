//! This module implements the greedy best-first search algorithm.

use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::search::{
    search_engines::{
        SearchEngine, SearchNodeStatus, SearchResult, SearchSpace, SearchStatistics, StateId,
    },
    states::SparseStatePacker,
    Heuristic, SuccessorGenerator, Task,
};
use std::cmp::Reverse;

/// Greedy best-first search
pub struct GBFS {}

impl GBFS {
    pub fn new() -> Self {
        Self {}
    }
}

impl SearchEngine for GBFS {
    fn search(
        &mut self,
        task: &Task,
        generator: Box<dyn SuccessorGenerator>,
        mut heuristic: Box<dyn Heuristic>,
    ) -> (SearchResult, SearchStatistics) {
        let mut statistics = SearchStatistics::new();
        let packer = SparseStatePacker::new(task);
        let mut priority_queue = PriorityQueue::new();
        let mut search_space = SearchSpace::new(packer.pack(&task.initial_state));
        let root_node = search_space.get_root_node_mut();

        let heuristic = heuristic.as_mut();
        root_node.open(
            OrderedFloat(0.),
            heuristic.evaluate(&task.initial_state, task),
        );
        priority_queue.push(root_node.get_state_id(), Reverse(root_node.get_h()));

        if task.goal.is_satisfied(&task.initial_state) {
            return (SearchResult::Success(vec![]), statistics);
        }

        while !priority_queue.is_empty() {
            let sid = priority_queue.pop().unwrap().0;
            let node = search_space.get_node_mut(sid);

            if node.get_status() == SearchNodeStatus::Closed {
                continue;
            }
            node.close();
            let state_id = node.get_state_id();
            let g_value = node.get_g();
            statistics.increment_expanded_nodes();

            let state = packer.unpack(search_space.get_state(sid));
            if task.goal.is_satisfied(&state) {
                // We get the node again so that the borrow checker knows it is
                // immutable
                let goal_node = search_space.get_node(state_id);
                return (
                    SearchResult::Success(search_space.extract_plan(&goal_node)),
                    statistics,
                );
            }

            let mut successors = Vec::new();
            let mut actions = Vec::new();
            for action_schema in &task.action_schemas {
                let applicable_actions = generator.get_applicable_actions(&state, action_schema);
                statistics.increment_generated_actions(applicable_actions.len());
                for action in applicable_actions {
                    let successor = generator.generate_successor(&state, action_schema, &action);
                    successors.push(successor);
                    actions.push(action);
                }
            }
            statistics.increment_generated_actions(actions.len());
            let child_node_ids: Vec<StateId> = actions
                .into_iter()
                .zip(successors.iter())
                .map(|(action, successor)| {
                    let child_node =
                        search_space.insert_or_get_node(packer.pack(&successor), action, state_id);
                    child_node.get_state_id()
                })
                .collect();

            let mut states_to_evaluate = vec![];
            let mut new_nodes = vec![];
            let mut possibly_reopened_nodes = vec![];
            for (successor, child_node_id) in successors.into_iter().zip(child_node_ids.into_iter())
            {
                let child_node = search_space.get_node_mut(child_node_id);
                if child_node.get_status() == SearchNodeStatus::New {
                    states_to_evaluate.push(successor);
                    new_nodes.push(child_node.get_state_id());
                } else {
                    possibly_reopened_nodes.push(child_node.get_state_id());
                }
            }
            statistics.increment_generated_nodes(new_nodes.len());
            let h_values = heuristic.evaluate_batch(&states_to_evaluate, task);

            for (child_node_id, h_value) in new_nodes.into_iter().zip(h_values.into_iter()) {
                let child_node = search_space.get_node_mut(child_node_id);
                child_node.open(g_value + 1., h_value);
                statistics.increment_evaluated_nodes();
                priority_queue.push(child_node_id, Reverse(h_value));
            }

            for child_node_id in possibly_reopened_nodes.into_iter() {
                let child_node = search_space.get_node_mut(child_node_id);
                if g_value + 1. < child_node.get_g() {
                    child_node.open(g_value + 1., child_node.get_h());
                    statistics.increment_reopened_nodes();
                    priority_queue.push(child_node_id, Reverse(child_node.get_h()));
                }
            }
        }

        (SearchResult::ProvablyUnsolvable, statistics)
    }
}
