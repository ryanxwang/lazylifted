//! The State Change Learning Graph
use crate::{
    learning::graphs::{CGraph, NodeID},
    search::{Atom, DBState, Negatable, Object, PartialAction, Task},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

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
}

impl SclgCompiler {
    pub fn new(task: &Task) -> Self {
        let mut compiler = Self {
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            predicate_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(&self, _state: &DBState, _partial_action: &PartialAction) -> CGraph {
        todo!("Implement compile")
    }

    fn precompile(&mut self, task: &Task) {
        self.precompile_base_graph(task);

        todo!("Precompile other necessary data structures")
    }

    fn precompile_base_graph(&mut self, task: &Task) {
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
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, EnumCountMacro)]
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
