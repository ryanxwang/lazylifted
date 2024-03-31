use crate::search::action_schema::ActionSchema;
use crate::search::goal::Goal;
use crate::search::object::Object;
use crate::search::states::DBState;
use crate::{Domain, Name, Parser, Problem, Types};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use super::predicate::Predicate;

#[derive(Debug)]
pub struct Task {
    pub domain_name: Name,
    pub problem_name: Name,
    pub types: Types,
    pub objects: Vec<Object>,
    pub goal: Goal,
    pub initial_state: DBState,
    pub action_schemas: Vec<ActionSchema>,
    pub predicates: Vec<Predicate>,
    pub nullary_predicates: HashSet<usize>,
}

impl Task {
    pub fn from_path(domain_path: &PathBuf, problem_path: &PathBuf) -> Self {
        let domain_text =
            fs::read_to_string(domain_path).expect("Failed to read domain file, does it exist?");
        let problem_text =
            fs::read_to_string(problem_path).expect("Failed to read problem file, does it exist?");
        Self::from_text(&domain_text, &problem_text)
    }

    pub fn from_text(domain_text: &str, problem_text: &str) -> Self {
        let domain = Domain::from_str(domain_text).expect("Failed to parse domain file");
        let problem = Problem::from_str(problem_text).expect("Failed to parse problem file");

        assert_eq!(
            domain.name(),
            problem.domain(),
            "Problem domain does not match the domain."
        );

        // Build tables
        let predicate_table: HashMap<Name, usize> = domain
            .predicates()
            .iter()
            .enumerate()
            .map(|(index, predicate)| (predicate.name().clone(), index))
            .collect();
        let type_table: HashMap<Name, usize> = domain
            .types()
            .iter()
            .enumerate()
            .map(|(index, typed)| (typed.value().clone(), index))
            .collect();
        let object_table: HashMap<Name, usize> = problem
            .objects()
            .iter()
            .enumerate()
            .map(|(index, object)| (object.value().clone(), index))
            .collect();

        // Build the various components
        let mut predicates = vec![];
        let mut nullary_predicates = HashSet::new();
        for (index, predicate) in domain.predicates().iter().enumerate() {
            if predicate.variables().is_empty() {
                nullary_predicates.insert(index);
            }

            predicates.push(Predicate::new(index, predicate, &type_table));
        }

        let action_schemas = domain
            .actions()
            .iter()
            .enumerate()
            .map(|(index, action)| {
                ActionSchema::new(index, action, &predicate_table, &type_table, &object_table)
            })
            .collect();

        let objects = problem
            .objects()
            .iter()
            .enumerate()
            .map(|(index, object)| Object::new(index, object, &type_table))
            .collect();

        let goal = Goal::new(problem.goals(), &predicate_table, &object_table);

        Self {
            domain_name: domain.name().clone(),
            problem_name: problem.name().clone(),
            types: domain.types().clone(),
            objects,
            goal,
            initial_state: DBState::from_problem(&problem, &predicate_table, &object_table),
            predicates,
            nullary_predicates,
            action_schemas,
        }
    }

    pub fn objects_per_type(&self) -> Vec<Vec<usize>> {
        let mut objects_per_type = vec![vec![]; self.types.values().len()];

        for object in &self.objects {
            for &type_index in &object.types {
                objects_per_type[type_index].push(object.index);
            }
        }

        objects_per_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    /// Test to make sure any change to the task translation is noticed.
    #[test]
    fn blocksworld() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        // human readable debug output
        // println!("{:#?}", task);
        assert_eq!(
            format!("{:?}", task),
            r#"Task { domain_name: Name(blocksworld), problem_name: Name(blocksworld-13), types: Types(TypedList([Typed(Name(object), Exactly(PrimitiveType(Name(object))))])), objects: [Object { index: 0, types: [0] }, Object { index: 1, types: [0] }, Object { index: 2, types: [0] }, Object { index: 3, types: [0] }], goal: Goal { atoms: [GoalAtom { predicate_index: 0, arguments: [3], negated: false }, GoalAtom { predicate_index: 4, arguments: [3, 1], negated: false }, GoalAtom { predicate_index: 4, arguments: [1, 2], negated: false }, GoalAtom { predicate_index: 4, arguments: [2, 0], negated: false }, GoalAtom { predicate_index: 1, arguments: [0], negated: false }], positive_nullary_goals: [], negative_nullary_goals: [] }, initial_state: DBState { relations: [Relation { predicate_symbol: 0, tuples: {[0]} }, Relation { predicate_symbol: 1, tuples: {[3]} }, Relation { predicate_symbol: 2, tuples: {} }, Relation { predicate_symbol: 3, tuples: {} }, Relation { predicate_symbol: 4, tuples: {[0, 1], [1, 2], [2, 3]} }], nullary_atoms: [false, false, true, false, false] }, action_schemas: [ActionSchema { name: ActionName(Name(pickup)), index: 0, parameters: [SchemaParameter { index: 0, type_index: 0 }], preconditions: [SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 1, negated: false, arguments: [Free(0)] }], positive_nullary_preconditions: [false, false, true, false, false], negative_nullary_preconditions: [false, false, false, false, false], effects: [SchemaAtom { predicate_index: 3, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 0, negated: true, arguments: [Free(0)] }, SchemaAtom { predicate_index: 1, negated: true, arguments: [Free(0)] }], positive_nullary_effects: [false, false, false, false, false], negative_nullary_effects: [false, false, true, false, false] }, ActionSchema { name: ActionName(Name(putdown)), index: 1, parameters: [SchemaParameter { index: 0, type_index: 0 }], preconditions: [SchemaAtom { predicate_index: 3, negated: false, arguments: [Free(0)] }], positive_nullary_preconditions: [false, false, false, false, false], negative_nullary_preconditions: [false, false, false, false, false], effects: [SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 1, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 3, negated: true, arguments: [Free(0)] }], positive_nullary_effects: [false, false, true, false, false], negative_nullary_effects: [false, false, false, false, false] }, ActionSchema { name: ActionName(Name(stack)), index: 2, parameters: [SchemaParameter { index: 0, type_index: 0 }, SchemaParameter { index: 1, type_index: 0 }], preconditions: [SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(1)] }, SchemaAtom { predicate_index: 3, negated: false, arguments: [Free(0)] }], positive_nullary_preconditions: [false, false, false, false, false], negative_nullary_preconditions: [false, false, false, false, false], effects: [SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 4, negated: false, arguments: [Free(0), Free(1)] }, SchemaAtom { predicate_index: 0, negated: true, arguments: [Free(1)] }, SchemaAtom { predicate_index: 3, negated: true, arguments: [Free(0)] }], positive_nullary_effects: [false, false, true, false, false], negative_nullary_effects: [false, false, false, false, false] }, ActionSchema { name: ActionName(Name(unstack)), index: 3, parameters: [SchemaParameter { index: 0, type_index: 0 }, SchemaParameter { index: 1, type_index: 0 }], preconditions: [SchemaAtom { predicate_index: 4, negated: false, arguments: [Free(0), Free(1)] }, SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(0)] }], positive_nullary_preconditions: [false, false, true, false, false], negative_nullary_preconditions: [false, false, false, false, false], effects: [SchemaAtom { predicate_index: 3, negated: false, arguments: [Free(0)] }, SchemaAtom { predicate_index: 0, negated: false, arguments: [Free(1)] }, SchemaAtom { predicate_index: 4, negated: true, arguments: [Free(0), Free(1)] }, SchemaAtom { predicate_index: 0, negated: true, arguments: [Free(0)] }], positive_nullary_effects: [false, false, false, false, false], negative_nullary_effects: [false, false, true, false, false] }], predicates: [Predicate { name: Name(clear), index: 0, arity: 1, types: [0] }, Predicate { name: Name(on-table), index: 1, arity: 1, types: [0] }, Predicate { name: Name(arm-empty), index: 2, arity: 0, types: [] }, Predicate { name: Name(holding), index: 3, arity: 1, types: [0] }, Predicate { name: Name(on), index: 4, arity: 2, types: [0, 0] }], nullary_predicates: {2} }"#
        );
    }
}
