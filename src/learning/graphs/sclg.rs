//! The State Change Learning Graph
use crate::{
    learning::graphs::{CGraph, Compiler2, NodeID},
    search::{
        successor_generators::SuccessorGeneratorName, ActionSchema, Atom, DBState, Negatable,
        Object, PartialAction, SuccessorGenerator, Task,
    },
};
use std::collections::{HashMap, HashSet};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

const NO_STATIC_PREDICATES: bool = true;

#[derive(Debug)]
pub struct SclgCompiler {
    /// Successor generator to use
    successor_generator: Box<dyn SuccessorGenerator>,
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
    pub fn new(task: &Task, successor_generator_name: SuccessorGeneratorName) -> Self {
        let mut compiler = Self {
            successor_generator: successor_generator_name.create(task),
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

        // TODO clean up this code, these should be typed as
        // Vec<Negatable<Atom>>
        let relevant_effects = self
            .successor_generator
            .get_applicable_actions(state, action_schema)
            .into_iter()
            .filter_map(|action| {
                let action = PartialAction::from(action.clone());
                if partial_action.is_superset_of(&action) {
                    Some(action_schema.partially_ground_effects(&action))
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut seen_nodes = HashSet::new();
        for atom in state.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            let node_id = match self.goal_atom_to_node_index.get(&atom) {
                Some(node_id) => {
                    let cur_node_type = Self::get_atom_type(graph[*node_id]);
                    graph[*node_id] =
                        Self::get_atom_colour(atom.predicate_index(), cur_node_type.as_achieved());
                    *node_id
                }
                None => {
                    let node_id = graph.add_node(Self::get_atom_colour(
                        atom.predicate_index(),
                        AtomType::new_state_atom(),
                    ));
                    for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                        let object_node_id = self.object_index_to_node_index[object_index];
                        graph.add_edge(node_id, object_node_id, arg_index as i32);
                    }
                    node_id
                }
            };

            for effect in &relevant_effects {
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

            for effect in &relevant_effects {
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
                AtomType::new_goal_atom(),
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
    fn get_atom_colour(predicate_index: usize, atom_type: AtomType) -> i32 {
        const START: i32 = 1;
        START + predicate_index as i32 * AtomType::count() as i32 + atom_type.into_repr()
    }

    #[inline(always)]
    fn get_atom_type(colour: i32) -> AtomType {
        const START: i32 = 1;
        AtomType::from_repr((colour - START) % AtomType::count() as i32).unwrap()
    }
}

impl Compiler2<DBState, PartialAction> for SclgCompiler {
    fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        self.compile(state, partial_action)
    }
}

#[allow(clippy::enum_variant_names)]
struct AtomType {
    atom_node_type: AtomNodeType,
    atom_change_type: AtomChangeType,
}

impl AtomType {
    #[inline(always)]
    pub const fn new_state_atom() -> Self {
        Self {
            atom_node_type: AtomNodeType::NonGoal,
            atom_change_type: AtomChangeType::Unchanged,
        }
    }

    #[inline(always)]
    pub const fn new_goal_atom() -> Self {
        Self {
            atom_node_type: AtomNodeType::UnachievedGoal,
            atom_change_type: AtomChangeType::Unchanged,
        }
    }

    #[inline(always)]
    pub fn as_achieved(&self) -> Self {
        Self {
            atom_node_type: self.atom_node_type.as_achieved(),
            atom_change_type: self.atom_change_type,
        }
    }

    #[inline(always)]
    pub fn as_added(&self) -> Self {
        Self {
            atom_node_type: self.atom_node_type,
            atom_change_type: self.atom_change_type.as_added(),
        }
    }

    #[inline(always)]
    pub fn as_removed(&self) -> Self {
        Self {
            atom_node_type: self.atom_node_type,
            atom_change_type: self.atom_change_type.as_removed(),
        }
    }

    pub const fn count() -> usize {
        AtomNodeType::COUNT * AtomChangeType::COUNT
    }

    pub fn from_repr(repr: i32) -> Option<Self> {
        let atom_node_type = AtomNodeType::from_repr(repr / AtomChangeType::COUNT as i32)?;
        let atom_change_type = AtomChangeType::from_repr(repr % AtomChangeType::COUNT as i32)?;
        Some(Self {
            atom_node_type,
            atom_change_type,
        })
    }

    pub fn into_repr(self) -> i32 {
        self.atom_node_type as i32 * AtomChangeType::COUNT as i32 + self.atom_change_type as i32
    }
}

#[derive(EnumCountMacro, Debug, Clone, Copy, FromRepr)]
#[repr(i32)]
enum AtomChangeType {
    Unchanged,
    Added,
    Removed,
    AddedAndRemoved,
}

impl AtomChangeType {
    #[inline(always)]
    pub fn as_added(&self) -> Self {
        match self {
            AtomChangeType::Unchanged => AtomChangeType::Added,
            AtomChangeType::Added => AtomChangeType::Added,
            AtomChangeType::Removed => AtomChangeType::AddedAndRemoved,
            AtomChangeType::AddedAndRemoved => AtomChangeType::AddedAndRemoved,
        }
    }

    #[inline(always)]
    pub fn as_removed(&self) -> Self {
        match self {
            AtomChangeType::Unchanged => AtomChangeType::Removed,
            AtomChangeType::Added => AtomChangeType::AddedAndRemoved,
            AtomChangeType::Removed => AtomChangeType::Removed,
            AtomChangeType::AddedAndRemoved => AtomChangeType::AddedAndRemoved,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(EnumCountMacro, Debug, Clone, Copy, FromRepr)]
#[repr(i32)]
enum AtomNodeType {
    /// The node is a goal node but not in the current state and is not being
    /// added.
    UnachievedGoal,
    /// The node is a goal node and in the current state and is not being
    /// removed by the partial action.
    AchievedGoal,
    /// The node is not a goal node, but is in the current state.
    NonGoal,
}

impl AtomNodeType {
    #[inline(always)]
    pub fn as_achieved(&self) -> Self {
        match self {
            AtomNodeType::UnachievedGoal => AtomNodeType::AchievedGoal,
            AtomNodeType::AchievedGoal => AtomNodeType::AchievedGoal,
            AtomNodeType::NonGoal => AtomNodeType::NonGoal,
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
        let compiler = SclgCompiler::new(&task, SuccessorGeneratorName::FullReducer);

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
                        graph[compiler.goal_atom_to_node_index[atom]],
                        SclgCompiler::get_atom_colour(
                            atom.predicate_index(),
                            AtomType::new_goal_atom()
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = SclgCompiler::new(&task, SuccessorGeneratorName::FullReducer);

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
                // (on b2 b3) is an achieved goal
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[atom]],
                    SclgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomType {
                            atom_node_type: AtomNodeType::AchievedGoal,
                            atom_change_type: AtomChangeType::Unchanged
                        }
                    )
                )
            } else {
                assert_eq!(
                    graph[compiler.goal_atom_to_node_index[atom]],
                    SclgCompiler::get_atom_colour(
                        atom.predicate_index(),
                        AtomType::new_goal_atom()
                    )
                )
            }
        }
    }
}
