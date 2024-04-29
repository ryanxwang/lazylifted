//! The State Change Learning Graph
use crate::{
    learning::graphs::{CGraph, Compiler2, NodeID},
    search::{ActionSchema, Atom, DBState, Negatable, Object, PartialAction, Task},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

// TODO: like felipe said, static information is still helpful. Instead of just
// ignoring them for performance, we should find a way to take advantage of
// them, or perhaps increase wl iteration number
const NO_STATIC_PREDICATES: bool = true;

#[derive(Debug, Clone)]
pub struct SclgCompiler {
    /// A precompiled graph for the task.
    base_graph: Option<CGraph>,
    /// A map from object index to node index in the base graph.
    object_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from predicate index to node index in the base graph.
    predicate_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from the atoms in the goal to node index in the base graph.
    goal_atom_to_node_index: HashMap<Atom, NodeID>,
    /// A copy of the action schemas of the task
    action_schemas: Vec<ActionSchema>,
    /// The static predicates of the task
    static_predicates: HashSet<usize>,
}

impl SclgCompiler {
    pub fn new(task: &Task) -> Self {
        let mut compiler = Self {
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            predicate_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
            action_schemas: task.action_schemas().to_owned(),
            static_predicates: task.static_predicates(),
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        let mut graph = self.base_graph.clone().unwrap();
        let action_schema = &self.action_schemas[partial_action.schema_index()];
        let partial_effects = action_schema.partially_ground_effects(partial_action);

        let mut seen_nodes = HashSet::new();
        for atom in state.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            let node_id = match self.goal_atom_to_node_index.get(&atom) {
                Some(node_id) => {
                    graph[*node_id] =
                        Self::get_atom_colour(atom.predicate_index(), AtomNodeType::AchievedGoal);
                    *node_id
                }
                None => {
                    let node_id = graph.add_node(Self::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::NonGoal,
                    ));
                    for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                        let object_node_id = self.object_index_to_node_index[object_index];
                        graph.add_edge(node_id, object_node_id, arg_index as i32);
                    }
                    node_id
                }
            };

            for effect in &partial_effects {
                if effect.includes(&atom) {
                    let cur_type = Self::get_atom_type(graph[node_id]);
                    match effect {
                        Negatable::Positive(_) => {
                            graph[node_id] =
                                Self::get_atom_colour(atom.predicate_index(), cur_type.as_added())
                        }
                        Negatable::Negative(_) => {
                            graph[node_id] =
                                Self::get_atom_colour(atom.predicate_index(), cur_type.as_removed())
                        }
                    }
                }
            }

            seen_nodes.insert(node_id);
        }
        for (atom, node_index) in &self.goal_atom_to_node_index {
            if seen_nodes.contains(node_index) {
                continue;
            }

            for effect in &partial_effects {
                if effect.includes(atom) {
                    let cur_type = Self::get_atom_type(graph[*node_index]);
                    match effect {
                        Negatable::Positive(_) => {
                            graph[*node_index] =
                                Self::get_atom_colour(atom.predicate_index(), cur_type.as_added())
                        }
                        Negatable::Negative(_) => {
                            graph[*node_index] =
                                Self::get_atom_colour(atom.predicate_index(), cur_type.as_removed())
                        }
                    }
                }
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

        // Goal atoms
        for atom in task.goal.atoms() {
            let atom = match atom {
                Negatable::Positive(atom) => atom,
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported"),
            };
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            let node_id = graph.add_node(Self::get_atom_colour(
                atom.predicate_index(),
                AtomNodeType::UnachievedGoal,
            ));
            for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[object_index];
                graph.add_edge(node_id, object_node_id, arg_index as i32);
            }
            self.goal_atom_to_node_index.insert(atom.clone(), node_id);
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

    fn get_atom_type(colour: i32) -> AtomNodeType {
        const START: i32 = 1;
        AtomNodeType::from_repr((colour - START) % AtomNodeType::COUNT as i32).unwrap()
    }
}

impl Compiler2<DBState, PartialAction> for SclgCompiler {
    fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        self.compile(state, partial_action)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, EnumCountMacro, FromRepr)]
#[repr(i32)]
enum AtomNodeType {
    /// The node is a goal node but not in the current state and is not being
    /// added.
    UnachievedGoal,
    /// Same as [`AtomNodeType::UnachievedGoal`] but being added by the partial
    /// action.
    AddedUnachievedGoal,
    /// The node is a goal node and in the current state and is not being
    /// removed by the partial action.
    AchievedGoal,
    /// Same as [`AtomNodeType::AchievedGoal`] but being removed by the partial
    /// action.
    RemovedAchievedGoal,
    /// The node is not a goal node, but is in the current state.
    NonGoal,
    /// The node is not a goal node, but is in the current state and is being
    /// removed by the partial action.
    RemovedNonGoal,
}

impl AtomNodeType {
    pub fn as_achieved(&self) -> Self {
        match self {
            AtomNodeType::UnachievedGoal | AtomNodeType::AddedUnachievedGoal => {
                AtomNodeType::AchievedGoal
            }
            AtomNodeType::AchievedGoal => AtomNodeType::AchievedGoal,
            AtomNodeType::RemovedAchievedGoal => AtomNodeType::RemovedAchievedGoal,
            AtomNodeType::NonGoal => AtomNodeType::NonGoal,
            AtomNodeType::RemovedNonGoal => AtomNodeType::RemovedNonGoal,
        }
    }

    // TODO: reconsider this decision
    /// We assume the best case scenario, i.e. adding overrides removing
    pub fn as_added(&self) -> Self {
        match self {
            AtomNodeType::UnachievedGoal => AtomNodeType::AddedUnachievedGoal,
            AtomNodeType::AddedUnachievedGoal => AtomNodeType::AddedUnachievedGoal,
            AtomNodeType::AchievedGoal => AtomNodeType::AchievedGoal,
            AtomNodeType::RemovedAchievedGoal => AtomNodeType::AchievedGoal,
            AtomNodeType::NonGoal => AtomNodeType::NonGoal,
            AtomNodeType::RemovedNonGoal => AtomNodeType::NonGoal,
        }
    }

    /// Removing does not override adding
    pub fn as_removed(&self) -> Self {
        match self {
            AtomNodeType::UnachievedGoal => AtomNodeType::UnachievedGoal,
            AtomNodeType::AddedUnachievedGoal => AtomNodeType::AddedUnachievedGoal,
            AtomNodeType::AchievedGoal => AtomNodeType::RemovedAchievedGoal,
            AtomNodeType::RemovedAchievedGoal => AtomNodeType::RemovedAchievedGoal,
            AtomNodeType::NonGoal => AtomNodeType::RemovedNonGoal,
            AtomNodeType::RemovedNonGoal => AtomNodeType::RemovedNonGoal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn blocksworld_precomilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = SclgCompiler::new(&task);

        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 9);
        assert_eq!(graph.edge_count(), 8);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                SclgCompiler::get_object_colour(object)
            );
        }
        for atom in task.goal.atoms() {
            match atom {
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
                Negatable::Positive(atom) => {
                    assert!(compiler.goal_atom_to_node_index.contains_key(atom));
                    assert_eq!(
                        graph[compiler.goal_atom_to_node_index[&atom]],
                        SclgCompiler::get_atom_colour(
                            atom.predicate_index(),
                            AtomNodeType::UnachievedGoal
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = SclgCompiler::new(&task);

        let graph = compiler.compile(
            &task.initial_state,
            &PartialAction::from(task.action_schemas()[3].clone()),
        );

        assert_eq!(graph.node_count(), 14);
        assert_eq!(graph.edge_count(), 14);
        for atom in task.goal.atoms() {
            let atom = match atom {
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
                Negatable::Positive(atom) => atom,
            };
            assert!(compiler.goal_atom_to_node_index.contains_key(atom));
            if atom.predicate_index() == 4 && atom.arguments() == vec![1, 2] {
                // (on b2 b3) is a removed achieved goal
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[&atom]],
                    SclgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::RemovedAchievedGoal
                    )
                )
            } else if atom.predicate_index() == 0 {
                // clear is being added
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[&atom]],
                    SclgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::AddedUnachievedGoal
                    )
                )
            } else {
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[&atom]],
                    SclgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::UnachievedGoal
                    )
                )
            }
        }
    }
}
