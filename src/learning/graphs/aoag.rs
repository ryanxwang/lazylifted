//! The Action-Object-Atom Graph

use crate::{
    learning::graphs::{CGraph, NodeID},
    search::{
        successor_generators::SuccessorGeneratorName, Action, ActionSchema, Atom, DBState,
        Negatable, PartialAction, SuccessorGenerator, Task,
    },
};
use std::collections::{HashMap, HashSet};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

use super::PartialActionCompiler;

const NO_STATIC_PREDICATES: bool = true;
const OBJECTS_COLOURED_BY_STATIC_PREDICATES: bool = true;

#[derive(Debug)]
pub struct AoagCompiler {
    /// Successor generator to use
    successor_generator: Box<dyn SuccessorGenerator>,
    /// The base graph to use
    base_graph: Option<CGraph>,
    /// A map from object index to node index in the base graph
    object_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from goal atoms to node index in the base graph
    goal_atom_to_node_index: HashMap<Atom, NodeID>,
    /// A copy of the action schemas of the task
    action_schemas: Vec<ActionSchema>,
    /// The static predicates of the task
    static_predicates: HashSet<usize>,
    /// Object colours, which may depend on how static predicates apply to them,
    /// indexed by object index
    object_colours: Vec<usize>,
    /// The maximum number of possible object colours - this is domain dependent
    /// (so not a const unfortunately) but not instance dependent, so that
    /// colours mean the same thing across instances
    max_object_colours: usize,
}

impl AoagCompiler {
    pub fn new(task: &Task, successor_generator_name: SuccessorGeneratorName) -> Self {
        // Fix here
        let object_colours = task
            .object_static_information()
            .iter()
            .map(|static_predicates| {
                if OBJECTS_COLOURED_BY_STATIC_PREDICATES {
                    let mut colour: usize = 0;
                    for predicate_index in static_predicates {
                        // negative so that colours don't overlap
                        colour += 1 << predicate_index;
                    }
                    colour
                } else {
                    0
                }
            })
            .collect();
        // can't just use the maximum seen colour, as this needs to be instance
        // agnostic
        let max_object_colours = (2 << task.max_static_information_count()) - 1;

        let mut compiler = Self {
            successor_generator: successor_generator_name.create(task),
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
            action_schemas: task.action_schemas().to_owned(),
            static_predicates: task.static_predicates(),
            object_colours,
            max_object_colours,
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        let mut graph = self.base_graph.clone().unwrap();
        let action_schema = &self.action_schemas[partial_action.schema_index()];

        let actions: Vec<Action> = self
            .successor_generator
            .get_applicable_actions_from_partial(state, action_schema, partial_action);

        // in the special case that there is only one applicable action, we make
        // reasoning more direct by directly applying the action, otherwise, we
        // add the actions in the graph and have them reasoned about
        let state = if actions.len() == 1 {
            &self
                .successor_generator
                .generate_successor(state, action_schema, &actions[0])
        } else {
            for action in actions {
                let node_id = graph.add_node(self.get_action_colour(action.index));
                for (arg_index, object_index) in action.instantiation.iter().enumerate() {
                    let object_node_id = self.object_index_to_node_index[object_index];
                    graph.add_edge(node_id, object_node_id, arg_index);
                }
            }

            state
        };

        for atom in state.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }
            match self.goal_atom_to_node_index.get(&atom) {
                Some(node_id) => {
                    graph[*node_id] =
                        self.get_atom_colour(atom.predicate_index(), AtomType::AchievedGoal)
                }
                None => {
                    let node_id = graph
                        .add_node(self.get_atom_colour(atom.predicate_index(), AtomType::NonGoal));
                    for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                        let object_node_id = self.object_index_to_node_index[object_index];
                        graph.add_edge(node_id, object_node_id, arg_index);
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
                graph.add_node(self.get_object_colour(object.index)),
            );
        }

        for atom in task.goal.atoms() {
            if NO_STATIC_PREDICATES && self.static_predicates.contains(&atom.predicate_index()) {
                continue;
            }

            let atom = match atom {
                Negatable::Positive(atom) => atom,
                Negatable::Negative(_) => {
                    panic!("Negative atoms in goal are not supported by AOAG")
                }
            };

            let node_id = graph
                .add_node(self.get_atom_colour(atom.predicate_index(), AtomType::UnachievedGoal));
            for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[object_index];
                graph.add_edge(node_id, object_node_id, arg_index);
            }
            self.goal_atom_to_node_index.insert(atom.clone(), node_id);
        }

        self.base_graph = Some(graph);
    }

    #[inline(always)]
    fn get_object_colour(&self, object_index: usize) -> usize {
        self.object_colours[object_index]
    }

    #[inline(always)]
    fn get_action_colour(&self, schema_index: usize) -> usize {
        let start = self.max_object_colours + 1;
        start + schema_index
    }

    #[inline(always)]
    fn get_atom_colour(&self, predicate_index: usize, atom_type: AtomType) -> usize {
        let start = self.max_object_colours + 1 + self.action_schemas.len();
        start + predicate_index * AtomType::COUNT + atom_type as usize
    }
}

impl PartialActionCompiler for AoagCompiler {
    fn compile(&self, state: &DBState, partial_action: &PartialAction) -> CGraph {
        self.compile(state, partial_action)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(EnumCountMacro, Debug, Clone, Copy, FromRepr)]
#[repr(i32)]
enum AtomType {
    AchievedGoal,
    UnachievedGoal,
    NonGoal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn blocksworld_precompilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = AoagCompiler::new(&task, SuccessorGeneratorName::FullReducer);

        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 9);
        assert_eq!(graph.edge_count(), 8);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                compiler.get_object_colour(object.index)
            );
        }
        for atom in task.goal.atoms() {
            match atom {
                Negatable::Negative(_) => panic!("Negative goal atoms are not supported in ILG"),
                Negatable::Positive(atom) => {
                    assert!(compiler.goal_atom_to_node_index.contains_key(atom));
                    assert_eq!(
                        graph[compiler.goal_atom_to_node_index[atom]],
                        compiler.get_atom_colour(atom.predicate_index(), AtomType::UnachievedGoal)
                    );
                }
            }
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = AoagCompiler::new(&task, SuccessorGeneratorName::FullReducer);
        let successor_generator = SuccessorGeneratorName::FullReducer.create(&task);

        let state = task.initial_state.clone();
        let state = successor_generator.generate_successor(
            &state,
            &task.action_schemas()[3],
            &Action::new(3, vec![0, 1]),
        );
        let state = successor_generator.generate_successor(
            &state,
            &task.action_schemas()[1],
            &Action::new(1, vec![0]),
        );
        let state = successor_generator.generate_successor(
            &state,
            &task.action_schemas()[3],
            &Action::new(3, vec![1, 2]),
        );
        // state: (clear b1) (clear b3) (on-table b1) (on-table b4) (holding b2)
        // (on b3 b4)

        // partial: (stack ?ob ?underob), so that we can stack on top of either
        // b1 or b3
        let graph = compiler.compile(&state, &PartialAction::new(2, vec![]));

        assert_eq!(graph.node_count(), 16);
        assert_eq!(graph.edge_count(), 18);

        fn count_nodes_with_colour(graph: &CGraph, colour: usize) -> usize {
            graph
                .node_indices()
                .filter(|node_id| graph[*node_id] == colour)
                .count()
        }

        // objects: b1 b2 b3 b4
        assert_eq!(
            // objects have colour 0 as blocksworld has no static predicates
            count_nodes_with_colour(&graph, 0),
            4
        );

        // clear, unachieved goal: (clear b4)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(0, AtomType::UnachievedGoal)
            ),
            1
        );

        // clear, nongoal: (clear b1), (clear b3)
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_atom_colour(0, AtomType::NonGoal)),
            2
        );

        // on-table, nongoal: (on-table b4)
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_atom_colour(1, AtomType::NonGoal)),
            1
        );

        // on-table, achieved goal: (on-table b1)
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_atom_colour(1, AtomType::AchievedGoal)),
            1
        );

        // arm-empty: nongoal:
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_atom_colour(2, AtomType::NonGoal)),
            0
        );

        // on, nongoal: (on b3 b4)
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_atom_colour(4, AtomType::NonGoal)),
            1
        );

        // on, unachievd goal: (on b1 b4), (on b4 b2) (on b2 b3)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(4, AtomType::UnachievedGoal)
            ),
            3
        );

        // stack actions: (stack b2 b1), (stack b2 b3)
        assert_eq!(
            count_nodes_with_colour(&graph, compiler.get_action_colour(2)),
            2
        );
    }
}
