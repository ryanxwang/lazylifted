//! The Instance Learning Graph (ILG) is a coloured graph representation of a
//! state in a planning task.
//!
//! For more detail, see:
//!
//! Dillon Ze Chen and Felipe Trevizan and Sylvie Thiébaux. Return to Tradition:
//! Learning Reliable Heuristics with Classical Machine Learning. ICAPS 2024.
//!
//! This implementation is based on the original Python implementation at
//! https://github.com/DillonZChen/goose

use crate::search::{Object, Task};
use crate::{
    learning::graphs::{
        utils::{atoms_of_goal, atoms_of_state, Atom},
        CGraph, NodeID,
    },
    search::DBState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Colours of atom nodes in the ILG.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(i32)]
enum AtomNodeType {
    /// The node is a goal node but not in the current state.
    UnachievedGoal,
    /// The node is a goal node and in the current state.
    AchievedGoal,
    /// The node is not a goal node, but is in the current state.
    NonGoal,
}
const NUM_ATOM_NODE_TYPES: i32 = 3;

/// A compiler to convert states to ILGs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlgCompiler {
    base_graph: Option<CGraph>,
    object_index_to_node_index: HashMap<usize, NodeID>,
    goal_atom_to_node_index: HashMap<Atom, NodeID>,
}

impl IlgCompiler {
    pub fn new(task: &Task) -> Self {
        let mut compiler = Self {
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
        };

        compiler.precompile(task);

        compiler
    }

    fn get_object_colour(_object: &Object) -> i32 {
        0
    }

    fn get_atom_colour(predicate_index: usize, atom_type: AtomNodeType) -> i32 {
        1 + predicate_index as i32 * NUM_ATOM_NODE_TYPES + atom_type as i32
    }

    pub fn compile(&self, state: &DBState) -> CGraph {
        let mut graph = self
            .base_graph
            .as_ref()
            .expect("Must precompile before compiling")
            .clone();

        let atoms = atoms_of_state(state);
        for atom in atoms {
            match self.goal_atom_to_node_index.get(&atom) {
                Some(node_id) => {
                    graph[*node_id] = Self::get_atom_colour(atom.0, AtomNodeType::AchievedGoal)
                }
                None => {
                    let node_id =
                        graph.add_node(Self::get_atom_colour(atom.0, AtomNodeType::NonGoal));
                    for (arg_index, object_index) in atom.1.iter().enumerate() {
                        let object_node_id = self.object_index_to_node_index[&object_index];
                        graph.add_edge(node_id, object_node_id, arg_index as i32);
                    }
                }
            }
        }

        graph
    }

    /// Precompile a base graph for a task. This base graph is cloned and then
    /// modified to produce the ILG for each state.
    fn precompile(&mut self, task: &Task) {
        let mut graph = CGraph::new_undirected();

        for object in &task.objects {
            self.object_index_to_node_index.insert(
                object.index,
                graph.add_node(Self::get_object_colour(object)),
            );
        }

        for atom in atoms_of_goal(&task.goal) {
            let node_id =
                graph.add_node(Self::get_atom_colour(atom.0, AtomNodeType::UnachievedGoal));
            for (arg_index, object_index) in atom.1.iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[&object_index];
                graph.add_edge(node_id, object_node_id, arg_index as i32);
            }
            self.goal_atom_to_node_index.insert(atom, node_id);
        }

        self.base_graph = Some(graph);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn node_type_to_i32() {
        assert_eq!(AtomNodeType::UnachievedGoal as i32, 0);
        assert_eq!(AtomNodeType::AchievedGoal as i32, 1);
        assert_eq!(AtomNodeType::NonGoal as i32, 2);
    }

    #[test]
    fn get_atom_colour() {
        assert_eq!(
            IlgCompiler::get_atom_colour(0, AtomNodeType::UnachievedGoal),
            1
        );
        assert_eq!(
            IlgCompiler::get_atom_colour(0, AtomNodeType::AchievedGoal),
            2
        );
        assert_eq!(IlgCompiler::get_atom_colour(0, AtomNodeType::NonGoal), 3);
        assert_eq!(
            IlgCompiler::get_atom_colour(1, AtomNodeType::UnachievedGoal),
            4
        );
        assert_eq!(
            IlgCompiler::get_atom_colour(1, AtomNodeType::AchievedGoal),
            5
        );
        assert_eq!(IlgCompiler::get_atom_colour(1, AtomNodeType::NonGoal), 6);
    }

    #[test]
    fn blocksworld_precomilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = IlgCompiler::new(&task);

        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 9);
        assert_eq!(graph.edge_count(), 8);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                IlgCompiler::get_object_colour(object)
            );
        }
        for atom in atoms_of_goal(&task.goal) {
            assert!(compiler.goal_atom_to_node_index.contains_key(&atom));
            assert_eq!(
                graph[compiler.goal_atom_to_node_index[&atom]],
                IlgCompiler::get_atom_colour(atom.0, AtomNodeType::UnachievedGoal)
            );
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = IlgCompiler::new(&task);

        let graph = compiler.compile(&task.initial_state);

        assert_eq!(graph.node_count(), 14);
        assert_eq!(graph.edge_count(), 14);
        for atom in atoms_of_goal(&task.goal) {
            assert!(compiler.goal_atom_to_node_index.contains_key(&atom));
            if atom.0 == 4 && atom.1 == vec![1, 2] {
                // (on b2 b3) is an achieved goal
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[&atom]],
                    IlgCompiler::get_atom_colour(atom.0, AtomNodeType::AchievedGoal)
                )
            } else {
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[&atom]],
                    IlgCompiler::get_atom_colour(atom.0, AtomNodeType::UnachievedGoal)
                )
            }
        }
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                IlgCompiler::get_object_colour(object)
            );
        }
    }
}
