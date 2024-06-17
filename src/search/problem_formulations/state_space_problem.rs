use crate::search::{
    states::{SparsePackedState, SparseStatePacker},
    Action, DBState, Heuristic, Plan, SearchNode, SearchNodeStatus, SearchProblem, SearchSpace,
    SearchStatistics, StateId, SuccessorGenerator, Task,
};
use std::rc::Rc;

const REOPEN: bool = false;

#[derive(Debug)]
pub struct StateSpaceProblem {
    task: Rc<Task>,
    statistics: SearchStatistics,
    packer: SparseStatePacker,
    generator: Box<dyn SuccessorGenerator>,
    search_space: SearchSpace<SparsePackedState, Action>,
    heuristic: Box<dyn Heuristic<DBState>>,
}

impl StateSpaceProblem {
    /// Construct a new state space search problem. Will also open the root
    /// node.
    pub fn new(
        task: Rc<Task>,
        generator: Box<dyn SuccessorGenerator>,
        mut heuristic: Box<dyn Heuristic<DBState>>,
    ) -> Self {
        let mut statistics = SearchStatistics::new();
        let packer = SparseStatePacker::new(&task);
        let mut search_space = SearchSpace::new(packer.pack(&task.initial_state));

        let root_node = search_space.get_root_node_mut();
        let initial_heuristic = heuristic.evaluate(&task.initial_state, &task);
        statistics.register_heuristic_value(initial_heuristic);
        root_node.open((0.).into(), initial_heuristic);

        Self {
            task,
            statistics,
            generator,
            packer,
            search_space,
            heuristic,
        }
    }
}

impl SearchProblem<SparsePackedState, Action> for StateSpaceProblem {
    fn initial_state(&self) -> &SearchNode<Action> {
        self.search_space.get_root_node()
    }

    fn is_goal(&self, state_id: StateId) -> bool {
        let state = self.packer.unpack(self.search_space.get_state(state_id));
        self.task.goal.is_satisfied(&state)
    }

    fn expand(&mut self, state_id: StateId) -> Vec<&SearchNode<Action>> {
        let node: &SearchNode<Action> = {
            let node = self.search_space.get_node_mut(state_id);
            if node.get_status() == SearchNodeStatus::Closed {
                return vec![];
            }
            node.close();
            node
        };
        self.statistics.increment_expanded_nodes();
        let g_value = node.get_g();
        let h_value = node.get_h();
        self.statistics.register_heuristic_value(h_value);

        let state = self.packer.unpack(self.search_space.get_state(state_id));

        let (new_states, new_ids, ids_to_reopen) = {
            let mut new_states = Vec::new();
            let mut new_ids = Vec::new();
            let mut ids_to_reopen = Vec::new();

            for action_schema in self.task.action_schemas() {
                let actions = self.generator.get_applicable_actions(&state, action_schema);
                self.statistics.increment_generated_actions(actions.len());
                for action in actions {
                    let successor =
                        self.generator
                            .generate_successor(&state, action_schema, &action);
                    let child_node = self.search_space.insert_or_get_node(
                        self.packer.pack(&successor),
                        action.clone(),
                        state_id,
                    );

                    // Partition the children into new nodes and those that were
                    // already seen before (and hence might need to be
                    // reopened). This way we can reuse the heuristic values for
                    // those seen before and only compute new ones. This also
                    // allows batch evaluation of the heuristic.
                    if child_node.get_status() == SearchNodeStatus::New {
                        new_states.push(successor);
                        new_ids.push(child_node.get_state_id());
                    } else if REOPEN && g_value + 1. < child_node.get_g() {
                        child_node.update_parent(state_id, action);
                        ids_to_reopen.push(child_node.get_state_id());
                    }
                }
            }
            (new_states, new_ids, ids_to_reopen)
        };
        self.statistics.increment_generated_nodes(new_ids.len());

        let h_values = self.heuristic.evaluate_batch(&new_states, &self.task);

        for (child_node_id, child_h_value) in new_ids.iter().zip(h_values.into_iter()) {
            self.statistics.increment_evaluated_nodes();
            let child_node = self.search_space.get_node_mut(*child_node_id);
            child_node.open(g_value + 1., child_h_value);
        }

        for child_node_id in ids_to_reopen.iter() {
            let child_node = self.search_space.get_node_mut(*child_node_id);
            self.statistics.increment_reopened_nodes();
            child_node.open(g_value + 1., child_node.get_h());
        }

        let mut child_nodes: Vec<&SearchNode<Action>> = Vec::new();
        for child_node_id in new_ids.into_iter().chain(ids_to_reopen.into_iter()) {
            child_nodes.push(self.search_space.get_node(child_node_id));
        }

        child_nodes
    }

    fn extract_plan(&self, goal_id: StateId) -> Plan {
        self.statistics.finalise_search();
        self.search_space
            .extract_plan(self.search_space.get_node(goal_id))
    }
}
