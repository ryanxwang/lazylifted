//! The Action-Object-Atom Graph

use crate::{
    learning::graphs::{CGraph, ColourDictionary, Compiler, NodeID, PartialActionCompiler},
    search::{
        successor_generators::SuccessorGeneratorName, Action, ActionSchema, Atom, DBState,
        Negatable, PartialAction, SuccessorGenerator, Task, NO_PARTIAL,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct AoagConfig {
    pub ignore_static_atoms: bool,
    pub objects_coloured_by_static_information: bool,
}

#[derive(Debug)]
pub struct AoagCompiler {
    /// Successor generator to use
    successor_generator: Box<dyn SuccessorGenerator>,
    /// Configuration for the compiler
    config: AoagConfig,
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
    /// Names of predicates, for colour descriptions
    predicate_names: Vec<String>,
    /// Names of action schemas, for colour descriptions
    schema_names: Vec<String>,
    /// Names of object colours, for colour descriptions
    object_colour_names: HashMap<usize, String>,
}

impl AoagCompiler {
    pub fn new(
        task: &Task,
        successor_generator_name: SuccessorGeneratorName,
        config: &AoagConfig,
    ) -> Self {
        // map from predicate index to exponent
        let static_predicate_map: HashMap<usize, usize> = task
            .object_static_information_predicates()
            .iter()
            .enumerate()
            .map(|(i, &p)| (p, i))
            .collect();

        let max_object_colours = if config.objects_coloured_by_static_information {
            // can't just use the maximum seen colour, as this needs to be
            // instance agnostic, so we ask the task for the maximum number of
            // static predicates that could appear in any instance
            (2 << task.object_static_information_predicates().len()) - 1
        } else {
            0
        };

        let mut object_colours = vec![];
        let mut object_colour_names = HashMap::new();

        for static_predicates in task.object_static_information() {
            let colour = if config.objects_coloured_by_static_information {
                let mut colour: usize = 0;
                for predicate_index in static_predicates {
                    colour += 1 << static_predicate_map[predicate_index];
                }
                colour
            } else {
                0
            };
            assert!(colour <= max_object_colours);

            object_colours.push(colour);
            object_colour_names.insert(colour, {
                if config.objects_coloured_by_static_information {
                    let mut pred_names = vec![];
                    for predicate_index in static_predicates {
                        pred_names.push(task.predicates[*predicate_index].name.clone().to_string());
                    }
                    format!("({})", pred_names.join(" "))
                } else {
                    "()".to_string()
                }
            });
        }

        let predicate_names = task
            .predicates
            .iter()
            .map(|p| p.name.clone().to_string())
            .collect();

        let schema_names = task
            .action_schemas()
            .iter()
            .map(|s| s.name().clone().to_string())
            .collect();

        let mut compiler = Self {
            successor_generator: successor_generator_name.create(task),
            config: config.clone(),
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
            action_schemas: task.action_schemas().to_owned(),
            static_predicates: task.static_predicates(),
            object_colours,
            object_colour_names,
            max_object_colours,
            predicate_names,
            schema_names,
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(
        &self,
        state: &DBState,
        partial_action: &PartialAction,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph {
        let mut graph = self.base_graph.clone().unwrap();

        let state = if *partial_action == NO_PARTIAL {
            state
        } else {
            let action_schema = &self.action_schemas[partial_action.schema_index()];

            let actions: Vec<Action> = self
                .successor_generator
                .get_applicable_actions_from_partial(state, action_schema, partial_action);

            // in the special case that there is only one applicable action, we make
            // reasoning more direct by directly applying the action, otherwise, we
            // add the actions in the graph and have them reasoned about
            if actions.len() == 1 {
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
            }
        };

        for atom in state.atoms() {
            if self.config.ignore_static_atoms
                && self.static_predicates.contains(&atom.predicate_index())
            {
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

        if let Some(colour_dictionary) = colour_dictionary {
            for node in graph.node_indices() {
                colour_dictionary.insert(graph[node] as i32, self.colour_description(graph[node]));
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
            if self.config.ignore_static_atoms
                && self.static_predicates.contains(&atom.predicate_index())
            {
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

    #[inline(always)]
    fn colour_description(&self, colour: usize) -> String {
        if colour <= self.max_object_colours {
            format!("object {}", self.object_colour_names[&colour])
        } else if colour <= self.max_object_colours + self.action_schemas.len() {
            format!(
                "action {}",
                self.schema_names[colour - self.max_object_colours - 1]
            )
        } else {
            let start = self.max_object_colours + 1 + self.action_schemas.len();
            let predicate_index = (colour - start) / AtomType::COUNT;
            let atom_type =
                AtomType::from_repr((colour - start) as i32 % AtomType::COUNT as i32).unwrap();
            format!(
                "atom {} {}",
                self.predicate_names[predicate_index], atom_type
            )
        }
    }
}

impl PartialActionCompiler for AoagCompiler {
    fn compile(
        &self,
        state: &DBState,
        partial_action: &PartialAction,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph {
        self.compile(state, partial_action, colour_dictionary)
    }
}

impl Compiler<DBState> for AoagCompiler {
    fn compile(&self, state: &DBState, colour_dictionary: Option<&mut ColourDictionary>) -> CGraph {
        self.compile(state, &NO_PARTIAL, colour_dictionary)
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

impl Display for AtomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomType::AchievedGoal => write!(f, "achieved-goal"),
            AtomType::UnachievedGoal => write!(f, "unachieved-goal"),
            AtomType::NonGoal => write!(f, "non-goal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    fn test_config() -> AoagConfig {
        AoagConfig {
            ignore_static_atoms: true,
            objects_coloured_by_static_information: true,
        }
    }

    #[test]
    fn blocksworld_precompilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler =
            AoagCompiler::new(&task, SuccessorGeneratorName::FullReducer, &test_config());

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
        let compiler =
            AoagCompiler::new(&task, SuccessorGeneratorName::FullReducer, &test_config());
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
        let graph = compiler.compile(&state, &PartialAction::new(2, vec![]), None);

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
