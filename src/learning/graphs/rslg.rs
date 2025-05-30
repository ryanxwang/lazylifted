//! The Resulting State Learning Graph
use crate::{
    learning::graphs::{CGraph, ColourDictionary, Compiler, NodeID, PartialActionCompiler},
    search::{
        successor_generators::SuccessorGeneratorName, ActionSchema, Atom, DBState, Negatable,
        PartialAction, PartialEffects, SuccessorGenerator, Task, NO_PARTIAL,
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
pub struct RslgConfig {
    pub ignore_static_atoms: bool,
    pub objects_coloured_by_static_information: bool,
    pub use_edge_colours: bool,
    pub object_edges_from_static_information: bool,
}

#[derive(Debug)]
pub struct RslgCompiler {
    /// Successor generator to use
    successor_generator: Box<dyn SuccessorGenerator>,
    /// Configuration for the compiler
    config: RslgConfig,
    /// A precompiled graph for the task.
    base_graph: Option<CGraph>,
    /// A map from object index to node index in the base graph.
    object_index_to_node_index: HashMap<usize, NodeID>,
    /// The goal atoms of the task
    goal_atom: HashSet<Atom>,
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
    /// The names of the predicates, for colour descriptions
    predicate_names: Vec<String>,
    /// The names of the object colours, for colour descriptions
    object_colour_names: HashMap<usize, String>,
}

impl RslgCompiler {
    pub fn new(
        task: &Task,
        successor_generator_name: SuccessorGeneratorName,
        config: &RslgConfig,
    ) -> Self {
        let singular_static_predicate_map: HashMap<usize, usize> = task
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
                    colour += 1 << singular_static_predicate_map[predicate_index];
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

        let mut compiler = Self {
            successor_generator: successor_generator_name.create(task),
            config: config.clone(),
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            goal_atom: task
                .goal
                .atoms()
                .iter()
                .map(|atom| {
                    assert!(atom.is_positive());
                    atom.underlying().to_owned()
                })
                .collect(),
            action_schemas: task.action_schemas().to_owned(),
            static_predicates: task.static_predicates(),
            object_colours,
            max_object_colours,
            predicate_names,
            object_colour_names,
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

        let PartialEffects {
            unavoidable_effects,
            optional_effects,
        } = if *partial_action == NO_PARTIAL {
            PartialEffects {
                unavoidable_effects: HashSet::new(),
                optional_effects: HashSet::new(),
            }
        } else {
            let action_schema = &self.action_schemas[partial_action.schema_index()];
            let applicable_actions = self
                .successor_generator
                .get_applicable_actions_from_partial(state, action_schema, partial_action);
            partial_action.get_partial_effects(action_schema, &applicable_actions)
        };

        let (unavoidable_adds, unavoidable_deletes) = unavoidable_effects.into_iter().fold(
            (HashSet::new(), HashSet::new()),
            |(mut adds, mut deletes), effect| {
                match effect {
                    Negatable::Positive(atom) => {
                        adds.insert(atom);
                    }
                    Negatable::Negative(atom) => {
                        deletes.insert(atom);
                    }
                };
                (adds, deletes)
            },
        );

        let (optional_adds, optional_deletes) = optional_effects.into_iter().fold(
            (HashSet::new(), HashSet::new()),
            |(mut adds, mut deletes), effect| {
                match effect {
                    Negatable::Positive(atom) => {
                        adds.insert(atom);
                    }
                    Negatable::Negative(atom) => {
                        deletes.insert(atom);
                    }
                };
                (adds, deletes)
            },
        );

        let mut atoms: HashMap<Atom, AtomType> = HashMap::new();
        for atom in state.atoms() {
            // for unavoidably deleted atoms, treat them as deleted
            if unavoidable_deletes.contains(&atom) {
                continue;
            }
            atoms.insert(atom, AtomType::achieved_nongoal_atom());
        }
        // for unavoidably added atoms, treat them as in the current state
        for atom in unavoidable_adds {
            atoms.insert(atom, AtomType::achieved_nongoal_atom());
        }
        for atom in self.goal_atom.iter() {
            atoms
                .entry(atom.clone())
                .and_modify(|atom_type| {
                    *atom_type = atom_type.with_in_goal();
                })
                .or_insert(AtomType::unachieved_goal_atom());
        }
        for atom in optional_adds {
            atoms
                .entry(atom.clone())
                .and_modify(|atom_type| {
                    *atom_type = atom_type.with_optional_add();
                })
                .or_insert(AtomType::unachieved_nongoal_atom().with_optional_add());
        }
        for atom in optional_deletes {
            atoms.entry(atom.clone()).and_modify(|atom_type| {
                *atom_type = atom_type.with_optional_delete();
            });
        }

        for (atom, atom_type) in atoms {
            if self.config.ignore_static_atoms
                && self.static_predicates.contains(&atom.predicate_index())
            {
                continue;
            }
            let node_id = graph.add_node(self.get_atom_colour(atom.predicate_index(), atom_type));

            for (arg_index, object_index) in atom.arguments().iter().enumerate() {
                let object_node_id = self.object_index_to_node_index[object_index];
                graph.add_edge(
                    node_id,
                    object_node_id,
                    if self.config.use_edge_colours {
                        arg_index
                    } else {
                        0
                    },
                );
            }
        }

        if let Some(colour_dictionary) = colour_dictionary {
            for node in graph.node_indices() {
                colour_dictionary.insert(graph[node] as i32, self.colour_description(graph[node]));
            }
        }

        // TODO-soon: for hard-to-ground problems, we probably only need to care
        // about the connected component of the graph that includes goal atoms.
        // This could significantly reduce the size of the graphs without losing
        // much information. Although to be cautionary, it will likely not reduce the size of wl features much at all.

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

        // Unlike for static object colours, we don't need to worry about
        // collision between edge colours for edges between objects
        if self.config.object_edges_from_static_information {
            task.object_pair_static_information()
                .iter()
                .for_each(|(&(u, v), pred_indices)| {
                    let u_node = self.object_index_to_node_index[&u];
                    let v_node = self.object_index_to_node_index[&v];
                    let colour = pred_indices.iter().map(|&pred_index| 1 << pred_index).sum();

                    // when we don't use edge colours, maybe we should just
                    // colour these all the same, but then there is literally no
                    // point right?
                    graph.add_edge(u_node, v_node, colour);
                });
        }

        self.base_graph = Some(graph);
    }

    #[inline(always)]
    fn get_object_colour(&self, object_index: usize) -> usize {
        self.object_colours[object_index]
    }

    #[inline(always)]
    fn get_atom_colour(&self, predicate_index: usize, atom_type: AtomType) -> usize {
        let start = self.max_object_colours + 1;
        start + predicate_index * AtomType::COUNT + atom_type.into_repr()
    }

    #[inline(always)]
    fn colour_description(&self, colour: usize) -> String {
        if colour <= self.max_object_colours {
            format!("object {}", self.object_colour_names[&colour])
        } else {
            let start = self.max_object_colours + 1;
            let predicate_index = (colour - start) / AtomType::COUNT;
            let atom_type =
                AtomType::from_repr((colour - start) as i32 % AtomType::COUNT as i32).unwrap();
            format!(
                "atom {} {}",
                self.predicate_names[predicate_index], atom_type,
            )
        }
    }
}

impl PartialActionCompiler for RslgCompiler {
    fn compile(
        &self,
        state: &DBState,
        partial_action: &PartialAction,
        colour_dictionary: Option<&mut ColourDictionary>,
    ) -> CGraph {
        self.compile(state, partial_action, colour_dictionary)
    }
}

impl Compiler<DBState> for RslgCompiler {
    fn compile(&self, state: &DBState, colour_dictionary: Option<&mut ColourDictionary>) -> CGraph {
        self.compile(state, &NO_PARTIAL, colour_dictionary)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
struct AtomType {
    atom_goal_type: AtomGoalType,
    atom_status_type: AtomStatusType,
}

impl Display for AtomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.atom_goal_type, self.atom_status_type)
    }
}

impl AtomType {
    #[inline(always)]
    pub const fn achieved_nongoal_atom() -> Self {
        Self {
            atom_goal_type: AtomGoalType::NonGoal,
            atom_status_type: AtomStatusType::Achieved,
        }
    }

    #[inline(always)]
    pub const fn unachieved_goal_atom() -> Self {
        Self {
            atom_goal_type: AtomGoalType::Goal,
            atom_status_type: AtomStatusType::Unachieved,
        }
    }

    #[inline(always)]
    pub const fn unachieved_nongoal_atom() -> Self {
        Self {
            atom_goal_type: AtomGoalType::NonGoal,
            atom_status_type: AtomStatusType::Unachieved,
        }
    }

    #[inline(always)]
    pub fn with_in_goal(&self) -> Self {
        Self {
            atom_goal_type: AtomGoalType::Goal,
            atom_status_type: self.atom_status_type,
        }
    }

    #[inline(always)]
    pub fn with_optional_add(&self) -> Self {
        Self {
            atom_goal_type: self.atom_goal_type,
            atom_status_type: self.atom_status_type.with_optional_add(),
        }
    }

    #[inline(always)]
    pub fn with_optional_delete(&self) -> Self {
        Self {
            atom_goal_type: self.atom_goal_type,
            atom_status_type: self.atom_status_type.with_optional_delete(),
        }
    }

    pub const COUNT: usize = AtomGoalType::COUNT * AtomStatusType::COUNT;

    pub fn into_repr(self) -> usize {
        self.atom_goal_type as usize * AtomStatusType::COUNT + self.atom_status_type as usize
    }

    pub fn from_repr(repr: i32) -> Option<Self> {
        let atom_goal_type = AtomGoalType::from_repr(repr / AtomStatusType::COUNT as i32)?;
        let atom_status_type = AtomStatusType::from_repr(repr % AtomStatusType::COUNT as i32)?;
        Some(Self {
            atom_goal_type,
            atom_status_type,
        })
    }
}

#[derive(EnumCountMacro, Debug, Clone, Copy, FromRepr)]
#[repr(i32)]
enum AtomStatusType {
    Achieved,
    Unachieved,
    OptionalAdd,
    OptionalDelete,
}

impl Display for AtomStatusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomStatusType::Achieved => write!(f, "achieved"),
            AtomStatusType::Unachieved => write!(f, "unachieved"),
            AtomStatusType::OptionalAdd => write!(f, "optional-add"),
            AtomStatusType::OptionalDelete => write!(f, "optional-delete"),
        }
    }
}

impl AtomStatusType {
    #[inline(always)]
    pub fn with_optional_add(&self) -> Self {
        match self {
            AtomStatusType::Achieved => AtomStatusType::Achieved,
            AtomStatusType::Unachieved => AtomStatusType::OptionalAdd,
            AtomStatusType::OptionalAdd => AtomStatusType::OptionalAdd,
            AtomStatusType::OptionalDelete => {
                panic!("Cannot have both optional add and delete, adds only work for atoms not in state, and deletes only work for atoms in state")
            }
        }
    }

    #[inline(always)]
    pub fn with_optional_delete(&self) -> Self {
        match self {
            AtomStatusType::Unachieved => AtomStatusType::Unachieved,
            AtomStatusType::Achieved => AtomStatusType::OptionalDelete,
            AtomStatusType::OptionalAdd => {
                panic!("Cannot have both optional add and delete, adds only work for atoms not in state, and deletes only work for atoms in state")
            }
            AtomStatusType::OptionalDelete => AtomStatusType::OptionalDelete,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(EnumCountMacro, Debug, Clone, Copy, FromRepr)]
#[repr(i32)]
enum AtomGoalType {
    /// The node is a goal node.
    Goal,
    /// The node is not a goal node.
    NonGoal,
}

impl Display for AtomGoalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomGoalType::Goal => write!(f, "goal"),
            AtomGoalType::NonGoal => write!(f, "non-goal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::Action;
    use crate::test_utils::*;

    fn test_config() -> RslgConfig {
        RslgConfig {
            ignore_static_atoms: true,
            objects_coloured_by_static_information: true,
            use_edge_colours: true,
            object_edges_from_static_information: true,
        }
    }

    #[test]
    fn blocksworld_precompilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler =
            RslgCompiler::new(&task, SuccessorGeneratorName::FullReducer, &test_config());

        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 0);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            // colour zero as blocksworld has no static predicates
            assert_eq!(graph[compiler.object_index_to_node_index[&object.index]], 0);
        }
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler =
            RslgCompiler::new(&task, SuccessorGeneratorName::FullReducer, &test_config());
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
        assert_eq!(graph.edge_count(), 16);

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

        // clear, optionally delete nongoal: (clear b1) (clear b3)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler
                    .get_atom_colour(0, AtomType::achieved_nongoal_atom().with_optional_delete())
            ),
            2
        );

        // clear, unachieved goal: (clear b4)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(0, AtomType::unachieved_goal_atom())
            ),
            1
        );

        // clear, achieved nongoal: (clear b2)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(0, AtomType::achieved_nongoal_atom())
            ),
            1
        );

        // on-table, achieved non goal: (on-table b4)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(1, AtomType::achieved_nongoal_atom())
            ),
            1
        );

        // on-table, achieved goal: (on-table b1)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(1, AtomType::achieved_nongoal_atom().with_in_goal())
            ),
            1
        );

        // arm-empty, achieved nongoal: (arm-empty)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(2, AtomType::achieved_nongoal_atom())
            ),
            1
        );

        // on, achieved nongoal: (on b3 b4)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(4, AtomType::achieved_nongoal_atom())
            ),
            1
        );

        // on, optionally add nongoal: (on b2 b1)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler
                    .get_atom_colour(4, AtomType::unachieved_nongoal_atom().with_optional_add())
            ),
            1
        );

        // on, optionally add goal: (on b2 b3)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(4, AtomType::unachieved_goal_atom().with_optional_add())
            ),
            1
        );

        // on, unachieved goal: (on b3 b1), (on b4 b2)
        assert_eq!(
            count_nodes_with_colour(
                &graph,
                compiler.get_atom_colour(4, AtomType::unachieved_goal_atom())
            ),
            2
        );
    }
}
