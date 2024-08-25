use crate::search::{
    states::{SparsePackedState, SparseStatePacker},
    Action, DBState, Heuristic, HeuristicValue, NodeId, PartialAction, PartialActionDiff, Plan,
    SearchContext, SearchNode, SearchNodeStatus, SearchProblem, SearchSpace, SuccessorGenerator,
    Task, NO_PARTIAL,
};
use ordered_float::Float;
use std::{collections::HashSet, rc::Rc};

const REOPEN: bool = false;

#[derive(Debug)]
pub struct PartialActionProblem {
    task: Rc<Task>,
    context: SearchContext,
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
        let mut context = SearchContext::new();
        let packer = SparseStatePacker::new(&task);
        let mut search_space = SearchSpace::new((packer.pack(&task.initial_state), NO_PARTIAL));

        let root_node = search_space.get_root_node_mut();
        // it doesn't make sense to evaluate the initial state, so we just give
        // it a heuristic value of infinity
        let initial_heuristic = HeuristicValue::infinity();
        context.register_heuristic_value(initial_heuristic);
        root_node.open((0.).into(), initial_heuristic);

        Self {
            task,
            context,
            generator,
            packer,
            search_space,
            heuristic,
        }
    }

    fn get_transitions(&self, node_id: NodeId) -> HashSet<PartialActionDiff> {
        let (state, partial) = self.search_space.get_state(node_id);
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
        node_id: NodeId,
        transition: &PartialActionDiff,
    ) -> (DBState, PartialAction) {
        let (state, partial) = self.search_space.get_state(node_id);
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

    fn is_goal(&self, node_id: NodeId) -> bool {
        let (state, partial) = self.search_space.get_state(node_id);
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

    fn expand(&mut self, node_id: NodeId) -> Vec<&SearchNode<PartialActionDiff>> {
        let node = {
            let node = self.search_space.get_node_mut(node_id);
            if node.get_status() == SearchNodeStatus::Closed {
                return vec![];
            }
            node.close();
            node
        };
        self.context.increment_expanded_nodes();
        let g_value = node.get_g();
        let h_value = node.get_h();
        self.context.register_heuristic_value(h_value);

        let transitions = self.get_transitions(node_id);
        self.context.increment_generated_actions(transitions.len());

        if transitions.len() == 1 {
            self.context.increment_skipped_evaluations();
            let transition = transitions.iter().next().unwrap();
            let (new_state, new_partial) = self.apply_transition(node_id, transition);
            let child_node = self.search_space.insert_or_get_node(
                (self.packer.pack(&new_state), new_partial.clone()),
                *transition,
                node_id,
            );

            let child_id: Option<NodeId> = if child_node.get_status() == SearchNodeStatus::New {
                // In this case, we technically should evaluate and give the
                // children a heuristic value. But since we want to skip
                // evaluations, we just use the same value as its parent, i.e.
                // the current node.
                self.context.increment_generated_nodes(1);
                child_node.open(g_value + 1., h_value);
                Some(child_node.get_node_id())
            } else if REOPEN && g_value + 1. < child_node.get_g() {
                // We don't count this into the reopened nodes statistic, so
                // that the number of reopened nodes is not inflated.
                child_node.update_parent(node_id, *transition);
                child_node.open(g_value + 1., child_node.get_h());
                Some(child_node.get_node_id())
            } else {
                None
            };

            if let Some(child_id) = child_id {
                // Don't skip goals
                if self.is_goal(child_id) {
                    return vec![self.search_space.get_node(child_id)];
                } else {
                    return self.expand(child_id);
                }
            } else {
                return vec![];
            }
        }

        let (new_states, new_ids, ids_to_reopen) = {
            let mut new_states = Vec::new();
            let mut new_ids = Vec::new();
            let mut ids_to_reopen = Vec::new();

            for transition in transitions {
                let (new_state, new_partial) = self.apply_transition(node_id, &transition);
                // TODO-soon: it turns out we frequently run out of memory. My
                // suspicion is that we access nodes that have already been
                // visited before too many times and we keep swapping their
                // pages back into memory (that's a whole 4k of memory!), and
                // then not use it at all since we don't reopen. Let's try
                // insert only and don't get the node,
                let child_node = self.search_space.insert_or_get_node(
                    (self.packer.pack(&new_state), new_partial.clone()),
                    transition,
                    node_id,
                );

                if child_node.get_status() == SearchNodeStatus::New {
                    new_states.push((new_state, new_partial));
                    new_ids.push(child_node.get_node_id());
                } else if REOPEN && g_value + 1. < child_node.get_g() {
                    child_node.update_parent(node_id, transition);
                    ids_to_reopen.push(child_node.get_node_id());
                }
            }

            (new_states, new_ids, ids_to_reopen)
        };
        self.context.increment_generated_nodes(new_states.len());

        let h_values = self.heuristic.evaluate_batch(&new_states, &self.task);

        for (child_node_id, child_h_value) in new_ids.iter().zip(h_values) {
            self.context.increment_evaluated_nodes();
            let child_node = self.search_space.get_node_mut(*child_node_id);
            child_node.open(g_value + 1., child_h_value);
        }
        for child_node_id in ids_to_reopen.iter() {
            let child_node = self.search_space.get_node_mut(*child_node_id);
            self.context.increment_reopened_nodes();
            child_node.open(g_value + 1., child_node.get_h());
        }

        let mut child_nodes = Vec::with_capacity(new_ids.len() + ids_to_reopen.len());
        for child_node_id in new_ids.into_iter().chain(ids_to_reopen.into_iter()) {
            child_nodes.push(self.search_space.get_node(child_node_id));
        }

        child_nodes
    }

    fn extract_plan(&mut self, goal_id: NodeId) -> Plan {
        self.context.finalise();
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
        let problem = create_problem();
        let root_node = problem.initial_state();

        let transitions = problem.get_transitions(root_node.get_node_id());
        assert_eq!(transitions.len(), 1);
    }
}
