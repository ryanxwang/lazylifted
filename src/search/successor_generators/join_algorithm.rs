use std::collections::VecDeque;

use crate::search::database::{hash_join, Table};
use crate::search::states::GroundAtom;
use crate::search::{DBState, SchemaArgument, SchemaAtom};

pub struct PrecompiledActionData {
    /// The index of the action schema in the task
    pub action_index: usize,
    /// Whether the action is ground (i.e. has no variables)
    pub is_ground: bool,
    pub relevant_precondition_atoms: Vec<SchemaAtom>,
}

pub trait JoinAlgorithm {
    fn instantiate(&self, state: &DBState, data: &PrecompiledActionData) -> Table {
        if data.is_ground {
            panic!("Ground action schemas should not be instantiated")
        }

        let mut tables: VecDeque<Table> = match self.parse_precond_into_join_program(data, state) {
            Some(tables) => VecDeque::from(tables),
            None => return Table::EMPTY,
        };
        debug_assert_eq!(tables.len(), data.relevant_precondition_atoms.len());

        let mut working_table = tables.pop_front().unwrap();

        while let Some(table) = tables.pop_front() {
            hash_join(&mut working_table, &table);
            if working_table.tuples.is_empty() {
                return Table::EMPTY;
            }
        }

        working_table
    }

    /// Create the set of tables corresponding to the precondition of the given
    /// action.
    fn parse_precond_into_join_program(
        &self,
        data: &PrecompiledActionData,
        state: &DBState,
    ) -> Option<Vec<Table>> {
        let mut tables = Vec::new();
        for schema_atom in &data.relevant_precondition_atoms {
            let mut indices = Vec::new();
            let mut constants = Vec::new();
            get_indices_and_constants_in_precondition(schema_atom, &mut indices, &mut constants);
            let tuples = select_tuples(state, schema_atom, &constants);
            if tuples.is_empty() {
                return None;
            }
            tables.push(Table::new(tuples, indices));
        }

        Some(tables)
    }
}

fn get_indices_and_constants_in_precondition(
    atom: &SchemaAtom,
    indices: &mut Vec<i32>,
    constants: &mut Vec<usize>,
) {
    debug_assert!(indices.is_empty());
    debug_assert!(constants.is_empty());

    for (i, arg) in atom.arguments().iter().enumerate() {
        match arg {
            SchemaArgument::Constant(index) => {
                indices.push(-(*index as i32 + 1));
                constants.push(i)
            }
            SchemaArgument::Free(index) => indices.push(*index as i32),
        }
    }
}

/// Select only the tuples that match the constants of a partially grouneded
/// precondition.
fn select_tuples(state: &DBState, atom: &SchemaAtom, constants: &[usize]) -> Vec<GroundAtom> {
    let mut tuples = Vec::new();

    for tuple in &state.relations[atom.predicate_index()].tuples {
        let mut match_constants = true;
        for &constant in constants {
            debug_assert!(atom.argument(constant).is_constant());
            if tuple[constant] != atom.argument(constant).get_index() {
                match_constants = false;
                break;
            }
        }
        if match_constants {
            tuples.push(tuple.clone());
        }
    }

    tuples
}

pub struct NaiveJoinAlgorithm;

impl NaiveJoinAlgorithm {
    pub fn new() -> Self {
        NaiveJoinAlgorithm {}
    }
}

impl JoinAlgorithm for NaiveJoinAlgorithm {}

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
        let generator = JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), &task);

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
        let generator = JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), &task);

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
