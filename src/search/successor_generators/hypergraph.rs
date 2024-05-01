use std::collections::{HashMap, HashSet};

use crate::search::ActionSchema;

#[derive(Debug)]
pub struct Hypergraph {
    /// A list of hypernodes. Each entry corresponds to a unique argument in the
    /// action schema preconditions.
    pub(super) hypernodes: Vec<usize>,
    /// A list of hyperedges. Each entry corresponds to the free variables in
    /// a precondition.
    pub(super) hyperedges: Vec<HashSet<usize>>,
    /// A map from schema precondition arguments (hypernodes) to the number of
    /// times its corresponding variable appears in the action schema
    /// preconditions.
    pub(super) node_counters: HashMap<usize, usize>,
    /// A map from the indices of [`hypernodes`] to the indices of corresponding
    /// schema precondition arguments.
    pub(super) node_indices: HashMap<usize, usize>,
    /// A map from hyperedge indices to the corresponding schema precondition
    /// indices.
    pub(super) edges_to_preconds: HashMap<usize, usize>,
    /// The preconditions that are missing from the hypergraph, i.e. have no
    /// free variables.
    pub(super) missing_preconds: Vec<usize>,
}

impl Hypergraph {
    pub fn from_action_schema(action_schema: &ActionSchema) -> Self {
        let mut hypernodes = Vec::new();
        let mut hyperedges = Vec::new();
        let mut node_counters = HashMap::new();
        let mut node_indices = HashMap::new();
        let mut edges_to_preconds = HashMap::new();
        let mut missing_preconds = Vec::new();

        for (i, precond) in action_schema.preconditions().iter().enumerate() {
            if precond.is_nullary() {
                continue;
            }

            let mut free_variables = HashSet::new();
            for arg in precond.arguments() {
                // We parse constants to negative numbers, so we don't treat them
                // here.
                if arg.is_constant() {
                    continue;
                }

                let index = arg.get_index();
                free_variables.insert(index);
                if hypernodes.contains(&index) {
                    if let Some(c) = node_counters.get_mut(&index) {
                        *c += 1;
                    }
                } else {
                    node_indices.insert(index, hypernodes.len());
                    hypernodes.push(index);
                    node_counters.insert(index, 1);
                }
            }

            if !free_variables.is_empty() {
                edges_to_preconds.insert(hyperedges.len(), i);
                hyperedges.push(free_variables);
            } else {
                missing_preconds.push(i);
            }
        }

        Self {
            hypernodes,
            hyperedges,
            node_counters,
            node_indices,
            edges_to_preconds,
            missing_preconds,
        }
    }
}
