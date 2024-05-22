use crate::search::{
    states::{SparsePackedState, SparseStatePacker},
    Action, DBState, Heuristic, HeuristicValue, PartialAction, PartialActionDiff, Plan, SearchNode,
    SearchNodeStatus, SearchProblem, SearchSpace, SearchStatistics, StateId, SuccessorGenerator,
    Task, NO_PARTIAL,
};
use ordered_float::Float;
use std::{collections::HashSet, rc::Rc};

#[derive(Debug)]
pub struct PartialActionProblem {
    task: Rc<Task>,
    statistics: SearchStatistics,
    packer: SparseStatePacker,
    generator: Box<dyn SuccessorGenerator>,
    search_space: SearchSpace<(SparsePackedState, PartialAction), PartialActionDiff>,
    heuristic: Box<dyn Heuristic<(DBState, PartialAction)>>,
}

impl PartialActionProblem {
    pub fn new(
        task: Rc<Task>,
        generator: Box<dyn SuccessorGenerator>,
        heuristic: Box<dyn Heuristic<(DBState, PartialAction)>>,
    ) -> Self {
        let mut statistics = SearchStatistics::new();
        let packer = SparseStatePacker::new(&task);
        let mut search_space = SearchSpace::new((packer.pack(&task.initial_state), NO_PARTIAL));

        let root_node = search_space.get_root_node_mut();
        // it doesn't make sense to evaluate the initial state, so we just give
        // it a heuristic value of infinity
        let initial_heuristic = HeuristicValue::infinity();
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

    fn get_transitions(&mut self, state_id: StateId) -> HashSet<PartialActionDiff> {
        let (state, partial) = self.search_space.get_state(state_id);
        if *partial == NO_PARTIAL || partial.is_complete(&self.task) {
            let state = if *partial == NO_PARTIAL {
                self.packer.unpack(state)
            } else {
                let action = Action::from_partial(partial, &self.task).unwrap();
                let schema = &self.task.action_schemas()[action.index];
                let state = self.packer.unpack(state);
                self.generator.generate_successor(&state, schema, &action)
            };

            // This involves many redundant calls to generator actions,
            // optimise by caching? Not sure how valuable this is though.
            self.task
                .action_schemas()
                .iter()
                .filter_map(|schema| {
                    let actions = self.generator.get_applicable_actions(&state, schema);
                    if actions.is_empty() {
                        None
                    } else {
                        Some(PartialActionDiff::Schema(schema.index()))
                    }
                })
                .collect()
        } else {
            let state = self.packer.unpack(state);
            let schema = &self.task.action_schemas()[partial.schema_index()];
            let actions = self.generator.get_applicable_actions(&state, schema);

            let current_depth = partial.partial_instantiation().len();
            actions
                .iter()
                .filter_map(|action| {
                    let new_partial = PartialAction::from_action(action, current_depth + 1);
                    if partial.is_superset_of(&new_partial) {
                        Some(PartialActionDiff::Instantiation(
                            new_partial.partial_instantiation()[current_depth],
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    fn apply_transition(
        &self,
        state_id: StateId,
        transition: &PartialActionDiff,
    ) -> (DBState, PartialAction) {
        let (state, partial) = self.search_space.get_state(state_id);
        match transition {
            PartialActionDiff::Schema(schema_index) => {
                let new_state = if *partial == NO_PARTIAL {
                    self.packer.unpack(state)
                } else {
                    let action = Action::from_partial(partial, &self.task).unwrap();
                    let schema = &self.task.action_schemas()[action.index];
                    let state = self.packer.unpack(state);
                    self.generator.generate_successor(&state, schema, &action)
                };
                let new_partial = PartialAction::new(*schema_index, vec![]);
                (new_state, new_partial)
            }
            PartialActionDiff::Instantiation(object_index) => {
                let new_state = self.packer.unpack(state);
                let new_partial = partial.add_instantiation(*object_index);
                (new_state, new_partial)
            }
        }
    }
}

impl SearchProblem<(SparsePackedState, PartialAction), PartialActionDiff> for PartialActionProblem {
    fn initial_state(&self) -> &SearchNode<PartialActionDiff> {
        self.search_space.get_root_node()
    }

    fn is_goal(&self, state_id: StateId) -> bool {
        let (state, partial) = self.search_space.get_state(state_id);
        if *partial == NO_PARTIAL {
            let state = self.packer.unpack(state);
            return self.task.goal.is_satisfied(&state);
        }

        match Action::from_partial(partial, &self.task) {
            Some(action) => {
                let schema = &self.task.action_schemas()[action.index];
                let state = self.packer.unpack(state);
                let successor = self.generator.generate_successor(&state, schema, &action);
                self.task.goal.is_satisfied(&successor)
            }
            None => false,
        }
    }

    fn expand(&mut self, state_id: StateId) -> Vec<&SearchNode<PartialActionDiff>> {
        let node = {
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

        let transitions = self.get_transitions(state_id);
        self.statistics
            .increment_generated_actions(transitions.len());

        // If there is a unique transition (quite often the case), we can just
        // recurse on it to save a heuristic evaluation.
        if transitions.len() == 1 {
            let transition = transitions.iter().next().unwrap();
            let (new_state, new_partial) = self.apply_transition(state_id, transition);

            let child_id: Option<StateId> = {
                let child_node = self.search_space.insert_or_get_node(
                    (self.packer.pack(&new_state), new_partial.clone()),
                    *transition,
                    state_id,
                );

                // We give it the same heuristic value as the parent, as we
                // know it's the only child and want to avoid evaluating it.
                if child_node.get_status() == SearchNodeStatus::New {
                    self.statistics.increment_generated_nodes(1);
                    child_node.open(g_value + 1., h_value);
                    Some(child_node.get_state_id())
                } else if g_value + 1. < child_node.get_g() {
                    self.statistics.increment_reopened_nodes();
                    child_node.update_parent(state_id, *transition);
                    child_node.open(g_value + 1., child_node.get_h());
                    Some(child_node.get_state_id())
                } else {
                    None
                }
            };

            if let Some(child_id) = child_id {
                return self.expand(child_id);
            } else {
                return vec![];
            }
        }

        let (new_states, new_ids, ids_to_reopen) = {
            let mut new_states = Vec::new();
            let mut new_ids = Vec::new();
            let mut ids_to_reopen = Vec::new();

            for transition in transitions {
                let (new_state, new_partial) = self.apply_transition(state_id, &transition);
                let child_node = self.search_space.insert_or_get_node(
                    (self.packer.pack(&new_state), new_partial.clone()),
                    transition,
                    state_id,
                );

                if child_node.get_status() == SearchNodeStatus::New {
                    new_states.push((new_state, new_partial));
                    new_ids.push(child_node.get_state_id());
                } else if g_value + 1. < child_node.get_g() {
                    child_node.update_parent(state_id, transition);
                    ids_to_reopen.push(child_node.get_state_id());
                }
            }

            (new_states, new_ids, ids_to_reopen)
        };
        self.statistics.increment_generated_nodes(new_states.len());

        let h_values = self.heuristic.evaluate_batch(&new_states, &self.task);

        let mut found_improvement = false;
        for (child_node_id, child_h_value) in new_ids.iter().zip(h_values) {
            self.statistics.increment_evaluated_nodes();
            let child_node = self.search_space.get_node_mut(*child_node_id);
            child_node.open(g_value + 1., child_h_value);

            if child_h_value < h_value {
                found_improvement = true;
            }
        }
        if found_improvement {
            self.statistics.increment_improving_expansions();
        }
        for child_node_id in ids_to_reopen.iter() {
            let child_node = self.search_space.get_node_mut(*child_node_id);
            self.statistics.increment_reopened_nodes();
            child_node.open(g_value + 1., child_node.get_h());
        }

        let mut child_nodes = Vec::with_capacity(new_ids.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::heuristics::ZeroHeuristic;
    use crate::search::successor_generators::SuccessorGeneratorName;
    use crate::test_utils::*;

    fn create_problem() -> PartialActionProblem {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let heuristic: Box<dyn Heuristic<(DBState, PartialAction)>> =
            Box::new(ZeroHeuristic::new());
        let successor_generators =
            SuccessorGeneratorName::create(&SuccessorGeneratorName::FullReducer, &task);

        PartialActionProblem::new(task, successor_generators, heuristic)
    }

    #[test]
    fn test_get_transitions() {
        let mut problem = create_problem();
        let root_node = problem.initial_state();

        let transitions = problem.get_transitions(root_node.get_state_id());
        assert_eq!(transitions.len(), 1);
    }
}
