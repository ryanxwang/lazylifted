//! Breadth first search

use std::collections::VecDeque;

use crate::search::{
    search_engines::{SearchEngine, SearchNodeStatus, SearchResult, SearchSpace, SearchStatistics},
    states::SparseStatePacker,
    Heuristic, SuccessorGenerator, Task, Verbosity,
};

pub struct BFS {
    statistics: SearchStatistics,
}

impl BFS {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            statistics: SearchStatistics::new(verbosity),
        }
    }
}

impl SearchEngine for BFS {
    fn search(
        &mut self,
        task: &Task,
        generator: &impl SuccessorGenerator,
        _heuristic: &impl Heuristic,
    ) -> SearchResult {
        let packer = SparseStatePacker::new(task);
        let mut queue = VecDeque::new();
        let mut search_space = SearchSpace::new(packer.pack(&task.initial_state));
        let root_node = search_space.get_root_node();

        root_node.open_with_f(0.);
        queue.push_back(root_node.get_state_id());

        if task.goal.is_satisfied(&task.initial_state) {
            return SearchResult::Success;
        }

        while !queue.is_empty() {
            let sid = queue.pop_front().unwrap();
            let node = search_space.get_node(sid);

            if node.get_status() == SearchNodeStatus::Closed {
                continue;
            }
            node.close();
            let state_id = node.get_state_id();
            let f_value = node.get_f();
            self.statistics.increment_expanded_nodes();

            let state = packer.unpack(search_space.get_state(sid));

            for action_schema in &task.action_schemas {
                let actions = generator.get_applicable_actions(&state, action_schema);
                self.statistics.increment_generated_actions(actions.len());

                for action in actions {
                    let successor = generator.generate_successor(&state, action_schema, &action);
                    let child_node =
                        search_space.insert_or_get_node(packer.pack(&successor), action, state_id);
                    if child_node.get_status() == SearchNodeStatus::New {
                        child_node.open_with_f(f_value + 1.);
                        if task.goal.is_satisfied(&successor) {
                            return SearchResult::Success;
                        }
                        queue.push_back(child_node.get_state_id());
                    }
                }
            }
        }

        SearchResult::ProvablyUnsolvable
    }
}
