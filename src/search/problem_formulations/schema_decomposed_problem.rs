use crate::search::{
    states::{SchemaDecomposedState, SchemaOrInstantiation, SparsePackedState, SparseStatePacker},
    DBState, Heuristic, NodeId, Plan, SearchNode, SearchNodeStatus, SearchProblem, SearchSpace,
    SearchStatistics, SuccessorGenerator, Task,
};
use std::rc::Rc;

const REOPEN: bool = false;

#[derive(Debug)]
pub struct SchemaDecomposedProblem {
    task: Rc<Task>,
    statistics: SearchStatistics,
    packer: SparseStatePacker,
    generator: Box<dyn SuccessorGenerator>,
    search_space: SearchSpace<SchemaDecomposedState<SparsePackedState>, SchemaOrInstantiation>,
    /// A heuristic that can evaluate a state and a schema index to a heuristic
    /// value.
    heuristic: Box<dyn Heuristic<SchemaDecomposedState<DBState>>>,
}

impl SchemaDecomposedProblem {
    /// Construct a new state space search problem. Will also open the root
    /// node.
    pub fn new(
        task: Rc<Task>,
        generator: Box<dyn SuccessorGenerator>,
        mut heuristic: Box<dyn Heuristic<SchemaDecomposedState<DBState>>>,
    ) -> Self {
        let mut statistics = SearchStatistics::new();
        let packer = SparseStatePacker::new(&task);
        let mut search_space = SearchSpace::new(SchemaDecomposedState::without_schema(
            packer.pack(&task.initial_state),
        ));

        let root_node = search_space.get_root_node_mut();
        let initial_heuristic = heuristic.evaluate(
            &SchemaDecomposedState::without_schema(task.initial_state.clone()),
            &task,
        );
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

    fn get_transitions(&mut self, node_id: NodeId) -> Vec<SchemaOrInstantiation> {
        let schema_decomposed_state = self.search_space.get_state(node_id);
        let state = self.packer.unpack(schema_decomposed_state.state());
        let mut transitions = Vec::new();

        match schema_decomposed_state.schema() {
            Some(schema_index) => {
                let applicable_actions = self
                    .generator
                    .get_applicable_actions(&state, &self.task.action_schemas()[schema_index]);

                for action in applicable_actions {
                    transitions.push(SchemaOrInstantiation::Instantiation(action));
                }
            }
            None => {
                for (schema_index, schema) in self.task.action_schemas().iter().enumerate() {
                    let applicable_actions = self.generator.get_applicable_actions(&state, schema);
                    if !applicable_actions.is_empty() {
                        transitions.push(SchemaOrInstantiation::Schema(schema_index));
                    }
                }
            }
        }

        transitions
    }

    fn apply_transition(
        &self,
        node_id: NodeId,
        transition: &SchemaOrInstantiation,
    ) -> (DBState, Option<usize>) {
        let schema_decomposed_state = self.search_space.get_state(node_id);
        let state = self.packer.unpack(schema_decomposed_state.state());

        match transition {
            SchemaOrInstantiation::Schema(schema_index) => (state, Some(*schema_index)),
            SchemaOrInstantiation::Instantiation(action) => {
                let schema_index = schema_decomposed_state.schema().unwrap();
                let new_state = self.generator.generate_successor(
                    &state,
                    &self.task.action_schemas()[schema_index],
                    action,
                );
                (new_state, None)
            }
        }
    }
}

impl SearchProblem<SchemaDecomposedState<SparsePackedState>, SchemaOrInstantiation>
    for SchemaDecomposedProblem
{
    fn initial_state(&self) -> &SearchNode<SchemaOrInstantiation> {
        self.search_space.get_root_node()
    }

    fn is_goal(&self, node_id: NodeId) -> bool {
        let schema_decomposed_state = self.search_space.get_state(node_id);
        if schema_decomposed_state.schema().is_none() {
            self.task
                .goal
                .is_satisfied(&self.packer.unpack(schema_decomposed_state.state()))
        } else {
            false
        }
    }

    fn expand(&mut self, node_id: NodeId) -> Vec<&SearchNode<SchemaOrInstantiation>> {
        let node: &SearchNode<SchemaOrInstantiation> = {
            let node = self.search_space.get_node_mut(node_id);
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

        let transitions = self.get_transitions(node_id);
        self.statistics
            .increment_generated_actions(transitions.len());

        let (new_states, new_ids, ids_to_reopen) = {
            let mut new_states = Vec::new();
            let mut new_ids = Vec::new();
            let mut ids_to_reopen = Vec::new();

            for transition in transitions {
                let (new_state, schema_index) = self.apply_transition(node_id, &transition);
                let child_node = self.search_space.insert_or_get_node(
                    SchemaDecomposedState::new(self.packer.pack(&new_state), schema_index),
                    transition.clone(),
                    node_id,
                );

                if child_node.get_status() == SearchNodeStatus::New {
                    new_states.push(SchemaDecomposedState::new(new_state, schema_index));
                    new_ids.push(child_node.get_node_id());
                } else if REOPEN && g_value + 1. < child_node.get_g() {
                    child_node.update_parent(node_id, transition);
                    ids_to_reopen.push(child_node.get_node_id());
                }
            }

            (new_states, new_ids, ids_to_reopen)
        };
        self.statistics.increment_generated_nodes(new_states.len());

        let h_values = self.heuristic.evaluate_batch(&new_states, &self.task);

        for (child_node_id, child_h_value) in new_ids.iter().zip(h_values) {
            self.statistics.increment_evaluated_nodes();
            let child_node = self.search_space.get_node_mut(*child_node_id);
            child_node.open(g_value + 1., child_h_value);
        }

        for child_node_id in ids_to_reopen.iter() {
            let child_node = self.search_space.get_node_mut(*child_node_id);
            self.statistics.increment_reopened_nodes();
            child_node.open(g_value + 1., child_node.get_h());
        }

        let mut child_nodes = Vec::with_capacity(new_ids.len() + ids_to_reopen.len());
        for child_node_id in new_ids.iter().chain(ids_to_reopen.iter()) {
            child_nodes.push(self.search_space.get_node(*child_node_id));
        }

        child_nodes
    }

    fn extract_plan(&self, goal_id: NodeId) -> Plan {
        self.statistics.finalise_search();
        self.search_space
            .extract_plan(self.search_space.get_node(goal_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{heuristics::ZeroHeuristic, successor_generators::SuccessorGeneratorName},
        test_utils::{BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT},
    };

    fn create_problem() -> SchemaDecomposedProblem {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let heuristic: Box<dyn Heuristic<SchemaDecomposedState<DBState>>> =
            Box::new(ZeroHeuristic::new());
        let successor_generator =
            SuccessorGeneratorName::create(&SuccessorGeneratorName::FullReducer, &task);

        SchemaDecomposedProblem::new(task, successor_generator, heuristic)
    }

    #[test]
    fn test_get_transitions() {
        let mut problem = create_problem();
        let root_node = problem.search_space.get_root_node();

        let transitions = problem.get_transitions(root_node.get_node_id());
        assert_eq!(transitions.len(), 1);
    }
}
