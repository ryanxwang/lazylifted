//! The Instance Learning Graph (ILG) is a coloured graph representation of a
//! state in a planning task.
//!
//! For more detail, see:
//!
//! Dillon Ze Chen and Felipe Trevizan and Sylvie Thiébaux. Return to Tradition:
//! Learning Reliable Heuristics with Classical Machine Learning. ICAPS 2024.
//!
//! This implementation is based on the original Python implementation at
//! <https://github.com/DillonZChen/goose>

use crate::search::{Negatable, Object, PartialAction, Task};
use crate::{
    learning::graphs::{CGraph, ColourDictionary, Compiler, NodeID, PartialActionCompiler},
    search::{Atom, DBState},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

const NO_STATIC_PREDICATES: bool = true;

/// Colours of atom nodes in the ILG.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, EnumCountMacro, Copy, FromRepr)]
#[repr(i32)]
enum AtomNodeType {
    /// The node is a goal node but not in the current state.
    UnachievedGoal,
    /// The node is a goal node and in the current state.
    AchievedGoal,
    /// The node is not a goal node, but is in the current state.
    NonGoal,
}

impl Display for AtomNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomNodeType::UnachievedGoal => write!(f, "unachieved-goal"),
            AtomNodeType::AchievedGoal => write!(f, "achieved-goal"),
            AtomNodeType::NonGoal => write!(f, "non-goal"),
        }
    }
}

/// A compiler to convert states to ILGs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlgCompiler {
    base_graph: Option<CGraph>,
    object_index_to_node_index: HashMap<usize, NodeID>,
    goal_atom_to_node_index: HashMap<Atom, NodeID>,
    static_predicates: HashSet<usize>,
    predicate_names: Vec<String>,
}

impl IlgCompiler {
    pub fn new(task: &Task) -> Self {
        let mut compiler = Self {
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
            static_predicates: task.static_predicates(),
            predicate_names: task
                .predicates
                .iter()
                .map(|p| p.name.clone().to_string())
                .collect(),
        };

        compiler.precompile(task);

        compiler
    }

    #[inline(always)]
    fn get_object_colour(_object: &Object) -> usize {
        // TODO-someday different objects can have different initial colours
        // based on constants associated with them, such as if a child requires
        // gluten free or not in childsnack. This information is currently not
        // included as we don't include statics
        const START: usize = 0;
        START
    }

    #[inline(always)]
    fn get_atom_colour(predicate_index: usize, atom_type: AtomNodeType) -> usize {
        const START: usize = 1;
        START + predicate_index * AtomNodeType::COUNT + atom_type as usize
    }

    #[inline(always)]
    fn colour_description(&self, colour: usize) -> String {
        if colour == 0 {
            return "object".to_string();
        }
        let predicate_index = (colour - 1) / AtomNodeType::COUNT;
        let atom_type = AtomNodeType::from_repr((colour - 1) as i32 % AtomNodeType::COUNT as i32)
            .expect("Invalid colour");
        format!("{} {}", self.predicate_names[predicate_index], atom_type)
    }

    pub fn compile(
        &self,
        state: &DBState,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph {
        let mut graph = self
            .base_graph
            .as_ref()
            .expect("Must precompile before compiling")
            .clone();

        for atom in state.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            match self.goal_atom_to_node_index.get(&atom) {
                Some(node_id) => {
                    graph[*node_id] =
                        Self::get_atom_colour(atom.predicate_index(), AtomNodeType::AchievedGoal)
                }
                None => {
                    let node_id = graph.add_node(Self::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::NonGoal,
                    ));
                    for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                        let object_node_id = self.object_index_to_node_index[object_index];
                        graph.add_edge(node_id, object_node_id, arg_index);
                    }
                }
            }
        }

        if let Some(colour_dictionary) = colour_dictionary {
            for node in graph.node_indices() {
                colour_dictionary.insert(graph[node] as i32, self.colour_description(graph[node]));
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

        for atom in task.goal.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            let atom = match atom {
                Negatable::Positive(atom) => atom,
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
            };

            let node_id = graph.add_node(Self::get_atom_colour(
                atom.predicate_index(),
                AtomNodeType::UnachievedGoal,
            ));
            for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[object_index];
                graph.add_edge(node_id, object_node_id, arg_index);
            }
            self.goal_atom_to_node_index.insert(atom.clone(), node_id);
        }

        self.base_graph = Some(graph);
    }
}

impl Compiler<DBState> for IlgCompiler {
    fn compile(&self, state: &DBState, colour_dictionary: Option<&mut ColourDictionary>) -> CGraph {
        self.compile(state, colour_dictionary)
    }
}

/// We implement the [`Compiler2`] trait for [`IlgCompiler`] to allow it to be
/// used in also for [`crate::learning::models::PartialActionModel`], this
/// implementation ignores the partial action completely.
impl PartialActionCompiler for IlgCompiler {
    fn compile(
        &self,
        state: &DBState,
        _: &PartialAction,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph {
        self.compile(state, colour_dictionary)
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
    fn blocksworld_precompilation() {
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
        for atom in task.goal.atoms() {
            match atom {
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
                Negatable::Positive(atom) => {
                    assert!(compiler.goal_atom_to_node_index.contains_key(atom));
                    assert_eq!(
                        graph[compiler.goal_atom_to_node_index[atom]],
                        IlgCompiler::get_atom_colour(
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
        let compiler = IlgCompiler::new(&task);

        let graph = compiler.compile(&task.initial_state, None);

        assert_eq!(graph.node_count(), 14);
        assert_eq!(graph.edge_count(), 14);
        for atom in task.goal.atoms() {
            let atom = match atom {
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
                Negatable::Positive(atom) => atom,
            };
            assert!(compiler.goal_atom_to_node_index.contains_key(atom));
            if atom.predicate_index() == 4 && atom.arguments() == vec![1, 2] {
                // (on b2 b3) is an achieved goal
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[atom]],
                    IlgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::AchievedGoal
                    )
                )
            } else {
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[atom]],
                    IlgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomNodeType::UnachievedGoal
                    )
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
