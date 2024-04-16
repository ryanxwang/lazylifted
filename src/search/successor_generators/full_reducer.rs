//! This module contains a
//! [`crate::search::successor_generators::JoinAlgorithm`] implementation that
//! uses the full reducer algorithm. See the Ullman book for more information.
use std::{cmp::Ordering, collections::HashSet};

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
        let mut full_join_order = vec![vec![]; task.action_schemas.len()];
        let mut full_reducer_program = vec![vec![]; task.action_schemas.len()];

        for action_schema in &task.action_schemas {
            let hypergraph = Hypergraph::from_action_schema(action_schema);

            if hypergraph.hyperedges.len() <= 1 {
                if !hypergraph.hyperedges.is_empty() {
                    full_join_order[action_schema.index].push(0);
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
                        full_reducer_program[action_schema.index].push((i_precond, j_precond));
                        full_reducer_back.push((j_precond, i_precond));
                        full_join_order[action_schema.index].push(i_precond);
                        break;
                    }
                }

                if ear.is_none() {
                    break;
                }
            }

            while let Some((i_precond, j_precond)) = full_reducer_back.pop() {
                full_reducer_program[action_schema.index].push((i_precond, j_precond));
            }

            // Add all missing preconditions to the join order
            for &i in &hypergraph.missing_preconds {
                full_join_order[action_schema.index].push(i);
            }

            full_join_order[action_schema.index].reverse();

            // Add all hyperedges that were not removed to the join. If the
            // action schema is acyclic, then there should be only one left.
            let remaining: Vec<usize> = removed
                .iter()
                .enumerate()
                .filter_map(|(i, &b)| if b { None } else { Some(i) })
                .collect();
            if remaining.len() == 1 {
                full_join_order[action_schema.index]
                    .push(hypergraph.edges_to_preconds[&remaining[0]]);
            } else {
                let mut q = PriorityQueue::new();
                full_join_order[action_schema.index] = Vec::with_capacity(
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
                    full_join_order[action_schema.index].push(precond);
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
    fn instantiate(&self, state: &DBState, data: &PrecompiledActionData) -> Table {
        if data.is_ground {
            panic!("Ground action schemas should not be instantiated")
        }

        let mut tables: Vec<Table> = match self.parse_precond_into_join_program(data, state) {
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
    use super::*;
    use crate::search::{
        successor_generators::{JoinSuccessorGenerator, SuccessorGenerator},
        Action, Task,
    };
    use crate::test_utils::*;

    #[test]
    fn applicable_actions_in_blocksworld_init() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let generator = JoinSuccessorGenerator::new(FullReducer::new(&task), &task);

        let state = &task.initial_state;

        // pickup is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[0]);
        assert_eq!(actions.len(), 0);

        // putdown is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[1]);
        assert_eq!(actions.len(), 0);

        // stack is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[2]);
        assert_eq!(actions.len(), 0);

        // unstack is the only applicable action in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].index, 3);
        assert_eq!(actions[0].instantiation, vec![0, 1]);
    }

    #[test]
    fn successor_generation_in_blocksworld() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let generator = JoinSuccessorGenerator::new(FullReducer::new(&task), &task);

        let mut states = Vec::new();
        states.push(task.initial_state);

        // action: (unstack b1 b2)
        let actions = generator.get_applicable_actions(&states[0], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[0], &task.action_schemas[3], &actions[0]));

        // state: (clear b2, on-table b4, holding b1, on b2 b3, on b3 b4)
        assert_eq!(
            format!("{}", states[1]),
            "(0 [1])(1 [3])(3 [0])(4 [1, 2])(4 [2, 3])"
        );

        // action: (putdown b1)
        let actions = generator.get_applicable_actions(&states[1], &task.action_schemas[1]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[1], &task.action_schemas[1], &actions[0]));

        // state: (clear b1, clear b2, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b4)
        assert_eq!(
            format!("{}", states[2]),
            "(0 [0])(0 [1])(1 [0])(1 [3])(4 [1, 2])(4 [2, 3])(2)"
        );

        // action: (unstack b2 b3)
        let actions = generator.get_applicable_actions(&states[2], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[2], &task.action_schemas[3], &actions[0]));

        // state: (clear b1, clear b3, on-table b1, on-table b4, holding b2, on b3 b4)
        assert_eq!(
            format!("{}", states[3]),
            "(0 [0])(0 [2])(1 [0])(1 [3])(3 [1])(4 [2, 3])"
        );

        // action: (putdown b2)
        let actions = generator.get_applicable_actions(&states[3], &task.action_schemas[1]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[3], &task.action_schemas[1], &actions[0]));

        // state: (clear b1, clear b2, clear b3, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b4)
        assert_eq!(
            format!("{}", states[4]),
            "(0 [0])(0 [1])(0 [2])(1 [0])(1 [1])(1 [3])(4 [2, 3])(2)"
        );

        // action: (unstack b3 b4)
        let actions = generator.get_applicable_actions(&states[4], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[4], &task.action_schemas[3], &actions[0]));

        // state: (clear b1, clear b2, clear b4, on-table b1, on-table b2, on-table b4, holding b3)
        assert_eq!(
            format!("{}", states[5]),
            "(0 [0])(0 [1])(0 [3])(1 [0])(1 [1])(1 [3])(3 [2])"
        );

        // action: (stack b3 b1)
        let actions = generator.get_applicable_actions(&states[5], &task.action_schemas[2]);
        assert_eq!(actions.len(), 3);
        assert!(actions.contains(&Action {
            // (stack b3 b1)
            index: 2,
            instantiation: vec![2, 0]
        }));
        assert!(actions.contains(&Action {
            // (stack b3 b2)
            index: 2,
            instantiation: vec![2, 1]
        }));
        assert!(actions.contains(&Action {
            // (stack b3 b4)
            index: 2,
            instantiation: vec![2, 3]
        }));
        let action = actions.iter().find(|a| a.instantiation[1] == 0).unwrap();
        states.push(generator.generate_successor(&states[5], &task.action_schemas[2], action));

        // state: (clear b2, clear b3, clear b4, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b1)
        assert_eq!(
            format!("{}", states[6]),
            "(0 [1])(0 [2])(0 [3])(1 [0])(1 [1])(1 [3])(4 [2, 0])(2)"
        );

        // action: (pickup b2)
        let actions = generator.get_applicable_actions(&states[6], &task.action_schemas[0]);
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&Action {
            // (pickup b2)
            index: 0,
            instantiation: vec![1]
        }));
        assert!(actions.contains(&Action {
            // (pickup b4)
            index: 0,
            instantiation: vec![3]
        }));
        let action = actions.iter().find(|a| a.instantiation[0] == 1).unwrap();
        states.push(generator.generate_successor(&states[6], &task.action_schemas[0], action));

        // state: (clear b3, clear b4, on-table b1, on-table b4, holding b2, on b3 b1)
        assert_eq!(
            format!("{}", states[7]),
            "(0 [2])(0 [3])(1 [0])(1 [3])(3 [1])(4 [2, 0])"
        );

        // action: (stack b2 b3)
        let actions = generator.get_applicable_actions(&states[7], &task.action_schemas[2]);
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&Action {
            // (stack b2 b3)
            index: 2,
            instantiation: vec![1, 2]
        }));
        assert!(actions.contains(&Action {
            // (stack b2 b4)
            index: 2,
            instantiation: vec![1, 3]
        }));
        let action = actions.iter().find(|a| a.instantiation[1] == 2).unwrap();
        states.push(generator.generate_successor(&states[7], &task.action_schemas[2], action));

        // state: (clear b2, clear b4, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b1)
        assert_eq!(
            format!("{}", states[8]),
            "(0 [1])(0 [3])(1 [0])(1 [3])(4 [1, 2])(4 [2, 0])(2)"
        );

        // action: (pickup b4)
        let actions = generator.get_applicable_actions(&states[8], &task.action_schemas[0]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[8], &task.action_schemas[0], &actions[0]));

        // state: (clear b2, on-table b1, holding b4, on b2 b3, on b3 b1)
        assert_eq!(
            format!("{}", states[9]),
            "(0 [1])(1 [0])(3 [3])(4 [1, 2])(4 [2, 0])"
        );

        // action: (stack b4 b2)
        let actions = generator.get_applicable_actions(&states[9], &task.action_schemas[2]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[9], &task.action_schemas[2], &actions[0]));

        // state: (clear b4, on-table b1, arm-empty, on b2 b3, on b3 b1, on b4 b2)
        assert_eq!(
            format!("{}", states[10]),
            "(0 [3])(1 [0])(4 [1, 2])(4 [2, 0])(4 [3, 1])(2)"
        );
    }
}
