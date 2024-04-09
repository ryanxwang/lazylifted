//! The Action Schema Learning Graph
use std::collections::HashMap;

use crate::{
    learning::graphs::{
        utils::{atoms_of_goal, atoms_of_state, Atom, SchemaPred},
        CGraph, NodeID,
    },
    search::{ActionSchema, DBState, Object, Predicate, SchemaArgument, SchemaParameter, Task},
};

#[derive(Debug, Clone)]
pub struct ASLGCompiler {
    /// A precompiled graph for the task.
    base_graph: Option<CGraph>,
    /// A map from object index to node index in the base graph.
    object_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from predicate index to node index in the base graph.
    predicate_index_to_node_index: HashMap<usize, NodeID>,
    /// A map from the atoms in the goal to node index in the base graph.
    goal_atom_to_node_index: HashMap<Atom, NodeID>,
    /// An action schema indexed vector of maps from schema predicates to their
    /// types.
    schema_pred_types: Vec<HashMap<SchemaPred, SchemaPredNodeType>>,
}

impl ASLGCompiler {
    pub fn new(task: &Task) -> Self {
        let mut compiler = Self {
            base_graph: None,
            object_index_to_node_index: HashMap::new(),
            predicate_index_to_node_index: HashMap::new(),
            goal_atom_to_node_index: HashMap::new(),
            schema_pred_types: vec![],
        };

        compiler.precompile(task);

        compiler
    }

    pub fn compile(&self, state: &DBState, action_schema: &ActionSchema) -> CGraph {
        let mut graph = self
            .base_graph
            .as_ref()
            .expect("Must precompile before compiling")
            .clone();

        for atom in atoms_of_state(state) {
            match self.goal_atom_to_node_index.get(&atom) {
                Some(&node_id) => {
                    graph[node_id] = Self::get_atom_colour(AtomNodeType::AchievedGoal);
                }
                None => {
                    let _node_id = self.create_atom_node(&mut graph, &atom, AtomNodeType::NonGoal);
                }
            }
        }

        let mut param_index_to_node_index = HashMap::new();
        for param in action_schema.parameters() {
            param_index_to_node_index
                .insert(param.index(), graph.add_node(Self::get_param_colour(param)));
        }

        for (schema_pred, &schema_pred_type) in self.schema_pred_types[action_schema.index].iter() {
            let node_id = graph.add_node(Self::get_schema_pred_colour(schema_pred_type));

            for (arg_index, &arg) in schema_pred.1.iter().enumerate() {
                match arg {
                    SchemaArgument::Free(param_index) => {
                        let param_node_id = param_index_to_node_index[&param_index];
                        graph.add_edge(node_id, param_node_id, arg_index as i32);
                    }
                    SchemaArgument::Constant(_object_index) => {
                        // TODO: for now we just ignore these. We very much
                        // could do something about them
                    }
                };
            }

            let pred_node_id = self.predicate_index_to_node_index[&schema_pred.0];
            graph.add_edge(node_id, pred_node_id, 0);
        }

        graph
    }

    fn precompile(&mut self, task: &Task) {
        self.precompile_base_graph(task);

        for action_schema in &task.action_schemas {
            self.schema_pred_types
                .push(self.precompute_schema_pred_types(action_schema));
        }
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

        // Predicate nodes
        for pred in &task.predicates {
            self.predicate_index_to_node_index
                .insert(pred.index, graph.add_node(Self::get_predicate_colour(pred)));
        }

        // Goal atoms
        for atom in atoms_of_goal(&task.goal) {
            let node_id = self.create_atom_node(&mut graph, &atom, AtomNodeType::UnachievedGoal);
            self.goal_atom_to_node_index.insert(atom, node_id);
        }

        self.base_graph = Some(graph);
    }

    fn precompute_schema_pred_types(
        &self,
        action_schema: &ActionSchema,
    ) -> HashMap<SchemaPred, SchemaPredNodeType> {
        let mut schema_pred_types = HashMap::new();

        // Deal with effects first
        for schema_atom in action_schema.effects() {
            let schema_pred: SchemaPred = (
                schema_atom.predicate_index(),
                schema_atom.arguments().into(),
            );
            assert!(schema_pred_types.contains_key(&schema_pred) == false);
            if schema_atom.is_negated() {
                schema_pred_types.insert(schema_pred, SchemaPredNodeType::Removed);
            } else {
                schema_pred_types.insert(schema_pred, SchemaPredNodeType::Added);
            }
        }
        for pred_index in action_schema
            .positive_nullary_effects()
            .iter()
            .enumerate()
            .filter_map(|(index, &pred)| if pred { Some(index) } else { None })
        {
            let schema_pred: SchemaPred = (pred_index, vec![]);
            assert!(schema_pred_types.contains_key(&schema_pred) == false);
            schema_pred_types.insert(schema_pred, SchemaPredNodeType::Added);
        }
        for pred_index in action_schema
            .negative_nullary_effects()
            .iter()
            .enumerate()
            .filter_map(|(index, &pred)| if pred { Some(index) } else { None })
        {
            let schema_pred: SchemaPred = (pred_index, vec![]);
            assert!(schema_pred_types.contains_key(&schema_pred) == false);
            schema_pred_types.insert(schema_pred, SchemaPredNodeType::Removed);
        }

        // Then deal with the preconditions not in the effects
        for schema_atom in action_schema.preconditions() {
            let schema_pred: SchemaPred = (
                schema_atom.predicate_index(),
                schema_atom.arguments().into(),
            );
            if schema_pred_types.contains_key(&schema_pred) {
                continue;
            }
            if schema_atom.is_negated() {
                schema_pred_types.insert(schema_pred, SchemaPredNodeType::RequiredFalse);
            } else {
                schema_pred_types.insert(schema_pred, SchemaPredNodeType::RequiredTrue);
            }
        }
        for pred_index in action_schema
            .positive_nullary_preconditions()
            .iter()
            .enumerate()
            .filter_map(|(index, &pred)| if pred { Some(index) } else { None })
        {
            let schema_pred: SchemaPred = (pred_index, vec![]);
            if schema_pred_types.contains_key(&schema_pred) {
                continue;
            }
            schema_pred_types.insert(schema_pred, SchemaPredNodeType::RequiredTrue);
        }
        for pred_index in action_schema
            .negative_nullary_preconditions()
            .iter()
            .enumerate()
            .filter_map(|(index, &pred)| if pred { Some(index) } else { None })
        {
            let schema_pred: SchemaPred = (pred_index, vec![]);
            if schema_pred_types.contains_key(&schema_pred) {
                continue;
            }
            schema_pred_types.insert(schema_pred, SchemaPredNodeType::RequiredFalse);
        }

        schema_pred_types
    }

    fn create_atom_node(
        self: &Self,
        graph: &mut CGraph,
        atom: &Atom,
        atom_type: AtomNodeType,
    ) -> NodeID {
        let node_id = graph.add_node(Self::get_atom_colour(atom_type));
        for (arg_index, object_index) in atom.1.iter().enumerate() {
            let object_node_id = self.object_index_to_node_index[&object_index];
            graph.add_edge(node_id, object_node_id, arg_index as i32);
        }
        let pred_node_id = self.predicate_index_to_node_index[&atom.0];
        graph.add_edge(node_id, pred_node_id, 0);
        node_id
    }

    fn get_object_colour(_object: &Object) -> i32 {
        const START: i32 = 0;
        START + 0
    }

    fn get_atom_colour(atom_type: AtomNodeType) -> i32 {
        const START: i32 = 1;
        START + atom_type as i32
    }

    fn get_schema_pred_colour(schema_pred_type: SchemaPredNodeType) -> i32 {
        const START: i32 = 1 + NUM_ATOM_NODE_TYPES;
        START + schema_pred_type as i32
    }

    fn get_param_colour(_param: &SchemaParameter) -> i32 {
        // TODO: consider using the index of the argument
        const START: i32 = 1 + NUM_ATOM_NODE_TYPES + NUM_SCHEMA_PRED_NODE_TYPES;
        START + 0
    }

    // The number of predicate colours is dependent on the domain, so for
    // simplicity we leave it last
    fn get_predicate_colour(pred: &Predicate) -> i32 {
        const START: i32 = 2 + NUM_ATOM_NODE_TYPES + NUM_SCHEMA_PRED_NODE_TYPES;
        START + pred.index as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(i32)]
enum SchemaPredNodeType {
    // This schema predicate is in the add list.
    Added,
    // This schema predicate is in the delete list.
    Removed,
    // This schema predicate is not in the effect, but is required to be in the
    // state by the precondition.
    RequiredTrue,
    // This schema predicate is not in the effect, but is required to not be in
    // the state by the precondition.
    RequiredFalse,
}
const NUM_SCHEMA_PRED_NODE_TYPES: i32 = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn blocksworld_precomilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = ASLGCompiler::new(&task);

        // Check the graph
        let graph = compiler.base_graph.as_ref().unwrap();
        assert_eq!(graph.node_count(), 14);
        assert_eq!(graph.edge_count(), 13);
        for object in &task.objects {
            assert!(compiler
                .object_index_to_node_index
                .contains_key(&object.index));
            assert_eq!(
                graph[compiler.object_index_to_node_index[&object.index]],
                ASLGCompiler::get_object_colour(object)
            );
        }
        for atom in atoms_of_goal(&task.goal) {
            assert!(compiler.goal_atom_to_node_index.contains_key(&atom));
            assert_eq!(
                graph[compiler.goal_atom_to_node_index[&atom]],
                ASLGCompiler::get_atom_colour(AtomNodeType::UnachievedGoal)
            );
        }
        for pred in &task.predicates {
            assert!(compiler
                .predicate_index_to_node_index
                .contains_key(&pred.index));
            assert_eq!(
                graph[compiler.predicate_index_to_node_index[&pred.index]],
                ASLGCompiler::get_predicate_colour(pred)
            );
        }

        // Check the schema pred types
        assert_eq!(compiler.schema_pred_types.len(), 4);
        assert_eq!(compiler.schema_pred_types[0].len(), 4);
        assert_eq!(compiler.schema_pred_types[1].len(), 4);
        assert_eq!(compiler.schema_pred_types[2].len(), 5);
        assert_eq!(compiler.schema_pred_types[3].len(), 5);
    }

    #[test]
    fn blocksworld_compilation() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let compiler = ASLGCompiler::new(&task);

        let state = task.initial_state.clone();
        let action_schema = task.action_schemas[0].clone();
        let graph = compiler.compile(&state, &action_schema);

        assert_eq!(graph.node_count(), 24);
        assert_eq!(graph.edge_count(), 31);
    }
}
