use std::collections::VecDeque;

use crate::search::database::{hash_join, Table};
use crate::search::states::GroundAtom;
use crate::search::{DBState, SchemaArgument, SchemaAtom};

pub struct PrecompiledActionData {
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
            Some(tables) => tables,
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
    ) -> Option<VecDeque<Table>> {
        let mut tables = VecDeque::new();
        for schema_atom in &data.relevant_precondition_atoms {
            let mut indices = Vec::new();
            let mut constants = Vec::new();
            get_indices_and_constants_in_precondition(schema_atom, &mut indices, &mut constants);
            let tuples = select_tuples(state, schema_atom, &constants);
            if tuples.is_empty() {
                return None;
            }
            tables.push_back(Table::new(tuples, indices));
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

    for (i, arg) in atom.arguments.iter().enumerate() {
        match arg {
            SchemaArgument::Constant(index) => {
                indices.push((*index as i32 + 1) * (-1));
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

    for tuple in &state.relations[atom.predicate_index].tuples {
        let mut match_constants = true;
        for &constant in constants {
            debug_assert!(atom.arguments[constant].is_constant());
            if tuple[constant] != atom.arguments[constant].get_index() {
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
