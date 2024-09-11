use crate::search::database::{hash_join, Table};
use crate::search::{
    raw_small_tuple, AtomSchema, DBState, Negatable, RawSmallTuple, SchemaArgument, SmallTuple,
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct PrecompiledActionData {
    /// The index of the action schema in the task
    pub action_index: usize,
    /// Whether the action is ground (i.e. has no variables)
    pub is_ground: bool,
    pub relevant_precondition_atoms: Vec<Negatable<AtomSchema>>,
    // [`objects_per_param[i]`] is the set of objects in for the type of the
    // `i`-th parameter of the action schema.
    pub objects_per_param: Vec<HashSet<usize>>,
}

pub trait JoinAlgorithm {
    fn instantiate(
        &self,
        state: &DBState,
        data: &PrecompiledActionData,
        // map from param index to object index
        fixed_schema_params: &HashMap<usize, usize>,
    ) -> Table {
        if data.is_ground {
            panic!("Ground action schemas should not be instantiated")
        }

        let mut tables: VecDeque<Table> =
            match self.parse_precond_into_join_program(data, state, fixed_schema_params) {
                Some(tables) => VecDeque::from(tables),
                None => return Table::EMPTY,
            };
        assert_eq!(tables.len(), data.relevant_precondition_atoms.len());

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
    /// action. For performance, we don't validate that the fixed_schema_params
    /// are actually valid (particularly the correct type for the parameter), so
    /// if not expect undefined behaviour.
    fn parse_precond_into_join_program(
        &self,
        data: &PrecompiledActionData,
        state: &DBState,
        fixed_schema_params: &HashMap<usize, usize>,
    ) -> Option<Vec<Table>> {
        let mut tables = Vec::new();
        for schema_atom in &data.relevant_precondition_atoms {
            let mut indices = Vec::new();
            let mut constants = Vec::new();
            let mut free_and_param_index = Vec::new();
            get_indices_and_constants_in_precondition(
                schema_atom,
                &mut indices,
                &mut constants,
                &mut free_and_param_index,
            );
            let tuples = select_tuples(
                state,
                schema_atom,
                &constants,
                &free_and_param_index,
                &data.objects_per_param,
                fixed_schema_params,
            );
            if tuples.is_empty() {
                return None;
            }
            tables.push(Table::new(tuples, indices));
        }

        Some(tables)
    }
}

fn get_indices_and_constants_in_precondition(
    atom: &Negatable<AtomSchema>,
    indices: &mut Vec<i32>,
    constants: &mut Vec<usize>,
    free_and_param_index: &mut Vec<(usize, usize)>,
) {
    assert!(indices.is_empty());
    assert!(constants.is_empty());
    assert!(free_and_param_index.is_empty());

    for (i, arg) in atom.arguments().iter().enumerate() {
        match arg {
            SchemaArgument::Constant(index) => {
                indices.push(-(*index as i32 + 1));
                constants.push(i);
            }
            SchemaArgument::Free(index) => {
                free_and_param_index.push((i, *index));
                indices.push(*index as i32);
            }
        }
    }
}

/// Select only the tuples that match the constants of a partially grouneded
/// precondition.
fn select_tuples(
    state: &DBState,
    atom: &Negatable<AtomSchema>,
    constants: &[usize],
    free_and_param_index: &[(usize, usize)],
    objects_per_param: &[HashSet<usize>],
    fixed_schema_params: &HashMap<usize, usize>,
) -> Vec<SmallTuple> {
    // TODO-soon: we spend a decent bit of time inside this closure (in fact,
    // that most of our time in the finding the applicable actions), can it be
    // faster?
    let tuple_matches = |tuple: &SmallTuple| -> bool {
        // the tuple matches the atom if
        // 1. when the atom is a constant, the tuple has the same value
        // 2. when the atom is a free variable, the type of the tuple is a
        //    subtype of the free variable
        let mut matches = true;
        for &constant in constants {
            assert!(atom.argument(constant).is_constant());
            if tuple[constant] != atom.argument(constant).get_index() {
                matches = false;
                break;
            }
        }
        if !matches {
            return false;
        }

        for &(free_index, param_index) in free_and_param_index {
            let object_index = tuple[free_index];
            if !objects_per_param[param_index].contains(&object_index) {
                matches = false;
                break;
            }
            if fixed_schema_params
                .get(&param_index)
                .is_some_and(|&expected_object_index| object_index != expected_object_index)
            {
                matches = false;
                break;
            }
        }
        matches
    };

    let mut tuples = Vec::new();

    if !atom.is_negated() {
        for tuple in &state.relations[atom.predicate_index()].tuples {
            if tuple_matches(tuple) {
                tuples.push(tuple.clone());
            }
        }
    } else {
        // For negative preconditions, we generate all the tuples that match the
        // the atom on constants and satisfy type requirements on free
        // variables. We then remove those tuples that are present in the
        // state.

        fn product(l: &HashSet<RawSmallTuple>, r: HashSet<usize>) -> HashSet<RawSmallTuple> {
            l.iter()
                .flat_map(|x| {
                    r.iter().map(move |y| {
                        let mut z = x.clone();
                        z.push(*y);
                        z
                    })
                })
                .collect()
        }
        let get_relevant_objects = |atom_index: usize| -> HashSet<usize> {
            if constants.contains(&atom_index) {
                HashSet::from([atom.argument(atom_index).get_index()])
            } else {
                let param_index = atom.argument(atom_index).get_index();
                if let Some(&object_index) = fixed_schema_params.get(&param_index) {
                    HashSet::from([object_index])
                } else {
                    objects_per_param[param_index].clone()
                }
            }
        };
        let mut all_tuples = get_relevant_objects(0)
            .iter()
            .map(|&x| raw_small_tuple![x])
            .collect::<HashSet<RawSmallTuple>>();
        for atom_index in 1..atom.arguments().len() {
            all_tuples = product(&all_tuples, get_relevant_objects(atom_index));
        }
        let all_tuples: HashSet<SmallTuple> = all_tuples.into_iter().map(SmallTuple::new).collect();

        for tuple in all_tuples {
            assert!(tuple_matches(&tuple));
            if state.relations[atom.predicate_index()]
                .tuples
                .contains(&tuple)
            {
                continue;
            }
            tuples.push(tuple);
        }
    }

    tuples
}

#[derive(Debug)]
pub struct NaiveJoinAlgorithm;

impl NaiveJoinAlgorithm {
    pub fn new() -> Self {
        NaiveJoinAlgorithm {}
    }
}

impl JoinAlgorithm for NaiveJoinAlgorithm {}

#[cfg(test)]
mod tests {
    use crate::search::successor_generators::{join_algorithm_tests::*, SuccessorGeneratorName};

    #[test]
    fn applicable_actions_in_blocksworld_init() {
        test_applicable_actions_in_blocksworld_init(SuccessorGeneratorName::Naive);
    }

    #[test]
    fn applicable_actions_from_partial_in_blocksworld() {
        test_applicable_actions_from_partial_in_blocksworld(SuccessorGeneratorName::Naive);
    }

    #[test]
    fn successor_generation_in_blocksworld() {
        test_successor_generation_in_blocksworld(SuccessorGeneratorName::Naive);
    }

    #[test]
    fn applicable_actions_in_spanner_init() {
        test_applicable_actions_in_spanner_init(SuccessorGeneratorName::Naive);
    }

    #[test]
    fn applicable_actions_in_ferry_init() {
        test_applicable_actions_in_ferry_init(SuccessorGeneratorName::Naive);
    }

    #[test]
    fn successor_generation_in_ferry() {
        test_successor_generation_in_ferry(SuccessorGeneratorName::Naive);
    }
}
