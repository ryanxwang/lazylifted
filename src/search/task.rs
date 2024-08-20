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
    /// The predicate indices of all nullary predicates.
    pub nullary_predicates: HashSet<usize>,
    /// The indices of objects per type. The index of the outer vector
    /// corresponds to the type index, the inner sets contain the indices of the
    /// objects of that type. These sets are not necessarily mutually disjoint
    /// due to subtyping.
    objects_per_type: Vec<HashSet<usize>>,
    /// The indices of static predicates that apply to an object. We only
    /// consider static predicate with a single argument.
    object_static_information: Vec<HashSet<usize>>,
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

        let action_schemas: Vec<ActionSchema> = domain
            .actions()
            .iter()
            .enumerate()
            .map(|(index, action)| {
                ActionSchema::new(index, action, &predicate_table, &type_table, &object_table)
            })
            .collect();

        let predicates = Self::mark_static_predicates(predicates, &action_schemas);

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

        let init_state = DBState::from_problem(&problem, &predicate_table, &object_table);

        let objects_per_type =
            Self::compute_objects_per_type(&type_table, domain.types(), &objects);

        let object_static_information =
            Self::compute_object_static_information(&init_state, &predicates, &objects);

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
            objects_per_type,
            object_static_information,
        }
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

    fn compute_object_static_information(
        init_state: &DBState,
        predicates: &[Predicate],
        object: &[Object],
    ) -> Vec<HashSet<usize>> {
        let mut object_static_information = vec![HashSet::new(); object.len()];

        for predicate in predicates {
            if predicate.arity != 1 || !predicate.is_static {
                continue;
            }

            for (object_index, _object) in object.iter().enumerate() {
                if init_state.relations[predicate.index]
                    .tuples
                    .iter()
                    .any(|tuple| tuple[0] == object_index)
                {
                    object_static_information[object_index].insert(predicate.index);
                }
            }
        }

        object_static_information
    }

    fn mark_static_predicates(
        predicates: Vec<Predicate>,
        action_schemas: &[ActionSchema],
    ) -> Vec<Predicate> {
        // basic determination of static predicates by simply checking if they
        // are used in any effect
        predicates
            .into_iter()
            .map(|mut predicate| {
                if !action_schemas.iter().any(|action| {
                    action
                        .effects()
                        .iter()
                        .any(|atom| atom.predicate_index() == predicate.index)
                }) {
                    predicate.mark_as_static();
                }
                predicate
            })
            .collect()
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

    pub fn object_static_information(&self) -> &[HashSet<usize>] {
        self.object_static_information.as_slice()
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
    fn static_predicate_detection_spanner() {
        let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);

        let static_predicates = task.static_predicates();

        // Only the "link" (3) predicate is static in spanner
        assert_eq!(static_predicates, HashSet::from([3]));
    }

    #[test]
    fn static_predicate_detection_childsnack() {
        let task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);

        let static_predicates = task.static_predicates();

        // In childsnack, predicates no_gluten_bread (3), no_gluten_content (4),
        // allergic_gluten(7), not_allrgic_gluten(8), and waiting (10) are
        // static.
        assert_eq!(static_predicates, HashSet::from([3, 4, 7, 8, 10]));
    }

    #[test]
    fn object_static_information() {
        let task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);

        let object_static_information = task.object_static_information();

        //  the 11 objects in the problem and then the constant kitchen
        assert_eq!(object_static_information.len(), 12);
        assert_eq!(
            Vec::from(object_static_information),
            vec![
                HashSet::from([7]), // child1 is allergic to gluten
                HashSet::from([8]), // child2 is not allergic to gluten
                HashSet::from([]),  // tray1
                HashSet::from([]),  // sandw1
                HashSet::from([]),  // sandw2
                HashSet::from([]),  // bread1
                HashSet::from([3]), // bread2 is gluten-free
                HashSet::from([]),  // content1
                HashSet::from([4]), // content2 is gluten-free
                HashSet::from([]),  // table1
                HashSet::from([]),  // table2
                HashSet::from([]),  // kitchen
            ]
        )
    }
}
