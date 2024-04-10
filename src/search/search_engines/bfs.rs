//! Breadth first search

use ordered_float::OrderedFloat;

use crate::search::{
    search_engines::{SearchEngine, SearchNodeStatus, SearchResult, SearchSpace, SearchStatistics},
    states::SparseStatePacker,
    Heuristic, PreferredOperator, SuccessorGenerator, Task,
};
use std::collections::VecDeque;

pub struct BFS {}

impl BFS {
    pub fn new() -> Self {
        Self {}
    }
}

impl SearchEngine for BFS {
    fn search(
        &mut self,
        task: &Task,
        generator: Box<dyn SuccessorGenerator>,
        _heuristic: Box<dyn Heuristic>,
        _preferred_operators: Option<Box<dyn PreferredOperator>>,
    ) -> (SearchResult, SearchStatistics) {
        let mut statistics = SearchStatistics::new();
        let packer = SparseStatePacker::new(task);
        let mut queue = VecDeque::new();
        let mut search_space = SearchSpace::new(packer.pack(&task.initial_state));
        let root_node = search_space.get_root_node_mut();

        root_node.open_with_f(OrderedFloat(0.));
        queue.push_back(root_node.get_state_id());

        if task.goal.is_satisfied(&task.initial_state) {
            return (SearchResult::Success(vec![]), statistics);
        }

        while !queue.is_empty() {
            let sid = queue.pop_front().unwrap();
            let node = search_space.get_node_mut(sid);

            if node.get_status() == SearchNodeStatus::Closed {
                continue;
            }
            node.close();
            let state_id = node.get_state_id();
            let f_value = node.get_f();
            statistics.increment_expanded_nodes();

            let state = packer.unpack(search_space.get_state(sid));

            for action_schema in &task.action_schemas {
                let actions = generator.get_applicable_actions(&state, action_schema);
                statistics.increment_generated_actions(actions.len());

                for action in actions {
                    let successor = generator.generate_successor(&state, action_schema, &action);
                    let child_node =
                        search_space.insert_or_get_node(packer.pack(&successor), action, state_id);
                    if child_node.get_status() == SearchNodeStatus::New {
                        child_node.open_with_f(f_value + 1.);
                        if task.goal.is_satisfied(&successor) {
                            // Annoying clone to satisfy the borrow checker
                            let goal_node = child_node.clone();
                            return (
                                SearchResult::Success(search_space.extract_plan(&goal_node)),
                                statistics,
                            );
                        }
                        queue.push_back(child_node.get_state_id());
                    }
                }
            }
        }

        (SearchResult::ProvablyUnsolvable, statistics)
    }
}
