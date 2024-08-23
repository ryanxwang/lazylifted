//! This module contains a
//! [`crate::search::successor_generators::JoinAlgorithm`] implementation that
//! uses the full reducer algorithm. See the Ullman book for more information.
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use priority_queue::PriorityQueue;

use crate::search::{
    database::{hash_join, semi_join, Table},
    successor_generators::{Hypergraph, JoinAlgorithm, PrecompiledActionData},
    DBState, Task,
};

/// The [`FullReducer`] struct contains the information needed to perform the
/// full reducer algorithm.
#[derive(Debug)]
pub struct FullReducer {
    full_join_order: Vec<Vec<usize>>,
    full_reducer_program: Vec<Vec<(usize, usize)>>,
}

impl FullReducer {
    /// On creation, the [`FullReducer`] computes which action schemas are
    /// acyclic or not. For the acyclic ones, it computes the full reducer
    /// program and the join order. For cyclic action schemas, it computes the
    /// "partial reducer".
    pub fn new(task: &Task) -> Self {
        let mut full_join_order = vec![vec![]; task.action_schemas().len()];
        let mut full_reducer_program = vec![vec![]; task.action_schemas().len()];

        for action_schema in task.action_schemas() {
            let hypergraph = Hypergraph::from_action_schema(action_schema);

            if hypergraph.hyperedges.len() <= 1 {
                if !hypergraph.hyperedges.is_empty() {
                    full_join_order[action_schema.index()].push(0);
                }
                continue;
            }

            // The GYO algorithm
            let mut full_reducer_back = Vec::new();
            let mut removed = vec![false; hypergraph.hyperedges.len()];
            loop {
                let mut ear = None;
                for i in 0..hypergraph.hyperedges.len() {
                    if removed[i] {
                        continue;
                    }

                    #[allow(clippy::needless_range_loop)] // slightly more readable
                    for j in 0..hypergraph.hyperedges.len() {
                        if removed[j] || i == j {
                            continue;
                        }

                        let diff = hypergraph.hyperedges[i]
                            .difference(&hypergraph.hyperedges[j])
                            .collect::<HashSet<_>>();

                        if diff.iter().all(|&n| hypergraph.node_counters[n] <= 1) {
                            ear = Some((i, j));
                            break;
                        }
                    }
                    if let Some((i, j)) = ear {
                        removed[i] = true;
                        let i_precond = hypergraph.edges_to_preconds[&i];
                        let j_precond = hypergraph.edges_to_preconds[&j];
                        full_reducer_program[action_schema.index()].push((i_precond, j_precond));
                        full_reducer_back.push((j_precond, i_precond));
                        full_join_order[action_schema.index()].push(i_precond);
                        break;
                    }
                }

                if ear.is_none() {
                    break;
                }
            }

            while let Some((i_precond, j_precond)) = full_reducer_back.pop() {
                full_reducer_program[action_schema.index()].push((i_precond, j_precond));
            }

            // Add all missing preconditions to the join order
            for &i in &hypergraph.missing_preconds {
                full_join_order[action_schema.index()].push(i);
            }

            full_join_order[action_schema.index()].reverse();

            // Add all hyperedges that were not removed to the join. If the
            // action schema is acyclic, then there should be only one left.
            let remaining: Vec<usize> = removed
                .iter()
                .enumerate()
                .filter_map(|(i, &b)| if b { None } else { Some(i) })
                .collect();
            if remaining.len() == 1 {
                full_join_order[action_schema.index()]
                    .push(hypergraph.edges_to_preconds[&remaining[0]]);
            } else {
                let mut q = PriorityQueue::new();
                full_join_order[action_schema.index()] = Vec::with_capacity(
                    hypergraph.hyperedges.len() + hypergraph.missing_preconds.len(),
                );
                for i in 0..hypergraph.hyperedges.len() {
                    q.push(
                        hypergraph.edges_to_preconds[&i],
                        hypergraph.hyperedges[i].len(),
                    );
                }
                for i in 0..hypergraph.missing_preconds.len() {
                    q.push(
                        hypergraph.missing_preconds[i],
                        action_schema.preconditions()[i].arguments().len(),
                    );
                }
                while let Some((precond, _)) = q.pop() {
                    full_join_order[action_schema.index()].push(precond);
                }
            }
        }

        Self {
            full_join_order,
            full_reducer_program,
        }
    }
}

impl JoinAlgorithm for FullReducer {
    fn instantiate(
        &self,
        state: &DBState,
        data: &PrecompiledActionData,
        fixed_schema_params: &HashMap<usize, usize>,
    ) -> Table {
        if data.is_ground {
            panic!("Ground action schemas should not be instantiated")
        }

        let mut tables: Vec<Table> =
            match self.parse_precond_into_join_program(data, state, fixed_schema_params) {
                Some(tables) => tables,
                None => return Table::EMPTY,
            };

        let order = &self.full_join_order[data.action_index];
        assert!(tables.len() == order.len());
        assert!(!tables.is_empty());

        for &(i, j) in &self.full_reducer_program[data.action_index] {
            let (table_i, table_j) = match i.cmp(&j) {
                Ordering::Less => {
                    let (left, right) = tables.split_at_mut(j);
                    (&mut left[i], &right[0])
                }
                Ordering::Equal => panic!("Cannot semi-join the same table"),
                Ordering::Greater => {
                    let (left, right) = tables.split_at_mut(i);
                    (&mut right[0], &left[j])
                }
            };
            let s = semi_join(table_i, table_j);
            if s == 0 {
                return Table::EMPTY;
            }
        }

        let mut working_table = tables[order[0]].clone();
        for i in 1..order.len() {
            hash_join(&mut working_table, &tables[order[i]]);
            if working_table.tuples.is_empty() {
                return Table::EMPTY;
            }
        }

        working_table
    }
}
#[cfg(test)]
mod tests {

    use crate::search::successor_generators::{join_algorithm_tests::*, SuccessorGeneratorName};

    #[test]
    fn applicable_actions_in_blocksworld_init() {
        test_applicable_actions_in_blocksworld_init(SuccessorGeneratorName::FullReducer);
    }

    #[test]
    fn successor_generation_in_blocksworld() {
        test_successor_generation_in_blocksworld(SuccessorGeneratorName::FullReducer);
    }

    #[test]
    fn applicable_actions_from_partial_in_blocksworld() {
        test_applicable_actions_from_partial_in_blocksworld(SuccessorGeneratorName::FullReducer);
    }

    #[test]
    fn applicable_actions_in_spanner_init() {
        test_applicable_actions_in_spanner_init(SuccessorGeneratorName::FullReducer);
    }

    #[test]
    fn applicable_actions_in_ferry_init() {
        test_applicable_actions_in_ferry_init(SuccessorGeneratorName::FullReducer);
    }

    #[test]
    fn successor_generation_in_ferry() {
        test_successor_generation_in_ferry(SuccessorGeneratorName::FullReducer);
    }
}
