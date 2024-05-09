//! The Resulting State Learning Graph
use crate::{
    learning::graphs::{CGraph, Compiler2, NodeID},
    search::{
        successor_generators::SuccessorGeneratorName, ActionSchema, Atom, DBState, Object,
        PartialAction, SuccessorGenerator, Task,
    },
};
use std::collections::{HashMap, HashSet};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

const NO_STATIC_PREDICATES: bool = true;

#[derive(Debug)]
pub struct RslgCompiler {
    /// Successor generator to use
    successor_generator: Box<dyn SuccessorGenerator>,
    /// A precompiled graph for the task.
    base_graph: Option<CGraph>,
    /// A map from object index to node index in the base graph.
    object_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from predicate index to node index in the base graph.
    predicate_index_to_node_index: HashMap<usize, NodeID>,
    /// The goal atoms of the task
    goal_atom: HashSet<Atom>,
    /// A copy of the action schemas of the task
    action_schemas: Vec<ActionSchema>,
    /// The static predicates of the task
    static_predicates: HashSet<usize>,
}

impl RslgCompiler {
    pub fn new(task: &Task, successor_generator_name: SuccessorGeneratorName) -> Self {
        let mut compiler = Self {
            successor_generator: successor_generator_name.create(task),
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            predicate_index_to_node_index: HashMap::new(),
            goal_atom: task
                .goal
                .atoms()
                .iter()
                .map(|atom| {
                    assert!(!atom.is_negated());
                    atom.underlying().to_owned()
                })
                .collect(),
            action_schemas: task.action_schemas().to_owned(),
            static_predicates: task.static_predicates(),
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        let mut graph = self.base_graph.clone().unwrap();
        let action_schema = &self.action_schemas[partial_action.schema_index()];

        let effects = partial_action.get_guaranteed_effects(action_schema);

        let mut removed_atoms_from_cur_state = HashSet::new();
        let mut atoms_in_new_state = HashSet::new();

        for effect in effects {
            let atom = effect.underlying().clone();
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            if effect.is_negated() {
                removed_atoms_from_cur_state.insert(atom);
            } else {
                atoms_in_new_state.insert(atom);
            }
        }
        for atom in state.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            if !removed_atoms_from_cur_state.contains(&atom) {
                atoms_in_new_state.insert(atom);
            }
        }

        let all_atoms = atoms_in_new_state
            .iter()
            .chain(self.goal_atom.iter())
            .collect::<HashSet<_>>();

        for atom in all_atoms {
            let atom_type = match (
                atoms_in_new_state.contains(atom),
                self.goal_atom.contains(atom),
            ) {
                (true, true) => AtomNodeType::AchievedGoal,
                (true, false) => AtomNodeType::NonGoal,
                (false, true) => AtomNodeType::UnachievedGoal,
                (false, false) => {
                    panic!("Atom is neither in the goal nor in the new state")
                }
            };

            let node_id = graph.add_node(Self::get_atom_colour(atom.predicate_index(), atom_type));
            for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[object_index];
                graph.add_edge(node_id, object_node_id, arg_index as i32);
            }
        }

        graph
    }

    fn precompile(&mut self, task: &Task) {
        let mut graph = CGraph::new_undirected();

        // Object nodes
        for object in &task.objects {
            self.object_index_to_node_index.insert(
                object.index,
                graph.add_node(Self::get_object_colour(object)),
            );
        }

        self.base_graph = Some(graph);
    }

    #[inline(always)]
    fn get_object_colour(_object: &Object) -> i32 {
        const START: i32 = 0;
        START
    }

    #[inline(always)]
    fn get_atom_colour(predicate_index: usize, atom_type: AtomNodeType) -> i32 {
        const START: i32 = 1;
        START + predicate_index as i32 * AtomNodeType::COUNT as i32 + atom_type as i32
    }
}

impl Compiler2<DBState, PartialAction> for RslgCompiler {
    fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        self.compile(state, partial_action)
    }
}

/// Colours of atom nodes in the RSLG.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, EnumCountMacro)]
#[repr(i32)]
enum AtomNodeType {
    /// The node is a goal node but not in the current state.
    UnachievedGoal,
    /// The node is a goal node and in the current state.
    AchievedGoal,
    /// The node is not a goal node, but is in the current state.
    NonGoal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn blocksworld_precomilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = RslgCompiler::new(&task, SuccessorGeneratorName::FullReducer);

        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 0);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                RslgCompiler::get_object_colour(object)
            );
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = RslgCompiler::new(&task, SuccessorGeneratorName::FullReducer);

        let state = task.initial_state.clone();
        let graph = compiler.compile(&state, &PartialAction::new(3, vec![]));

        assert_eq!(graph.node_count(), 13);
        assert_eq!(graph.edge_count(), 14);
    }
}
