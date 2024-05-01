use crate::parsed_types::{Domain, Name, Problem, Types};
use crate::parsers::Parser;
use crate::search::{ActionSchema, DBState, Goal, Object, Predicate};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Task {
    domain_name: Name,
    problem_name: Name,
    pub types: Types,
    pub objects: Vec<Object>,
    pub goal: Goal,
    pub initial_state: DBState,
    action_schemas: Vec<ActionSchema>,
    pub predicates: Vec<Predicate>,
    pub nullary_predicates: HashSet<usize>,
    objects_per_type: Vec<HashSet<usize>>,
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

        debug_assert_eq!(
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
        let mut object_table: HashMap<Name, usize> = problem
            .objects()
            .iter()
            .enumerate()
            .map(|(index, object)| (object.value().clone(), index))
            .collect();
        domain.constants().iter().for_each(|constant| {
            object_table.insert(constant.value().clone(), object_table.len());
        });

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

        let mut objects: Vec<Object> = problem
            .objects()
            .iter()
            .enumerate()
            .map(|(index, object)| Object::new(index, object, &type_table))
            .collect();
        domain
            .constants()
            .iter()
            .enumerate()
            .for_each(|(index, constant)| {
                objects.push(Object::new(
                    index + problem.objects().len(),
                    constant,
                    &type_table,
                ));
            });

        let goal = Goal::new(problem.goals(), &predicate_table, &object_table);

        let objects_per_type =
            Self::compute_objects_per_type(&type_table, domain.types(), &objects);

        let mut result = Self {
            domain_name: domain.name().clone(),
            problem_name: problem.name().clone(),
            types: domain.types().clone(),
            objects,
            goal,
            initial_state: DBState::from_problem(&problem, &predicate_table, &object_table),
            predicates,
            nullary_predicates,
            action_schemas,
            objects_per_type,
        };
        result.mark_static_predicates();

        result
    }

    fn parent_type(
        type_table: &HashMap<Name, usize>,
        types: &Types,
        type_index: usize,
    ) -> Option<usize> {
        let parent = types
            .get(type_index)?
            .type_()
            .get_primitive()
            .expect("Multiple parent types for a type are not supported")
            .name();
        type_table.get(parent).copied()
    }

    fn compute_objects_per_type(
        type_table: &HashMap<Name, usize>,
        types: &Types,
        objects: &[Object],
    ) -> Vec<HashSet<usize>> {
        let mut objects_per_type = vec![HashSet::new(); types.values().len()];

        for object in objects {
            for &type_index in &object.types {
                let mut type_index = type_index;
                loop {
                    objects_per_type[type_index].insert(object.index);
                    type_index = match Self::parent_type(type_table, types, type_index) {
                        Some(parent) => {
                            if parent == type_index {
                                break;
                            }
                            parent
                        }
                        None => break,
                    };
                }
            }
        }

        objects_per_type
    }

    fn mark_static_predicates(&mut self) {
        // basic determination of static predicates by simply checking if they
        // are used in any effect
        for predicate in &mut self.predicates {
            if !self.action_schemas.iter().any(|action| {
                action
                    .effects()
                    .iter()
                    .any(|atom| atom.predicate_index() == predicate.index)
            }) {
                predicate.mark_as_static();
            }
        }
    }

    pub fn domain_name(&self) -> &str {
        &self.domain_name
    }

    pub fn problem_name(&self) -> &str {
        &self.problem_name
    }

    pub fn action_schemas(&self) -> &[ActionSchema] {
        self.action_schemas.as_slice()
    }

    pub fn objects_per_type(&self) -> &[HashSet<usize>] {
        self.objects_per_type.as_slice()
    }

    pub fn static_predicates(&self) -> HashSet<usize> {
        self.predicates
            .iter()
            .filter(|predicate| predicate.is_static)
            .map(|predicate| predicate.index)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn blocksworld() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);

        assert_eq!(task.domain_name, "blocksworld");
        assert_eq!(task.problem_name, "blocksworld-13");
        assert_eq!(task.types.len(), 1);
        assert_eq!(task.objects.len(), 4);
        assert_eq!(task.goal.atoms().len(), 5);
        assert_eq!(task.initial_state.atoms().len(), 6);
        assert_eq!(task.action_schemas.len(), 4);
        assert_eq!(task.predicates.len(), 5);
        assert_eq!(task.nullary_predicates.len(), 1);
        assert_eq!(task.objects_per_type.len(), 1);
    }

    #[test]
    fn objects_per_type_spanner() {
        let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);

        assert_eq!(task.objects_per_type.len(), 5);
        // locations
        assert_eq!(
            task.objects_per_type[0],
            HashSet::from([7, 8, 9, 10, 11, 12, 13, 14])
        );
        // locatables
        assert_eq!(
            task.objects_per_type[1],
            HashSet::from([0, 1, 2, 3, 4, 5, 6])
        );
        // man
        assert_eq!(task.objects_per_type[2], HashSet::from([0]));
        // nut
        assert_eq!(task.objects_per_type[3], HashSet::from([5, 6]));
        // spanner
        assert_eq!(task.objects_per_type[4], HashSet::from([1, 2, 3, 4]));
    }

    #[test]
    fn static_predicate_detection() {
        let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);

        assert_eq!(task.predicates.len(), 6);
        assert!(!task.predicates[0].is_static); // at, not static
        assert!(!task.predicates[1].is_static); // carrying, not static
        assert!(!task.predicates[2].is_static); // usable, not static
        assert!(task.predicates[3].is_static); // link, static
        assert!(!task.predicates[4].is_static); // tightened, not static
        assert!(!task.predicates[5].is_static); // loose, not static
    }
}
