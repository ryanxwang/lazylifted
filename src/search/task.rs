use crate::parsed_types::{Domain, Name, Problem, Types};
use crate::parsers::Parser;
use crate::search::{
    heuristics::Requirement, remove_equalities, states::Relation, ActionSchema, DBState, Goal,
    Object, Predicate, RawSmallTuple, SmallTuple,
};
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tracing::info;

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
    /// The indices of static predicates that apply to a pair of objects. We
    /// only consider static predicates with two arguments. The key is a pair of
    /// object indices (sorted) and the value is the set of predicate indices
    /// that apply to that pair.
    object_pair_static_information: HashMap<(usize, usize), HashSet<usize>>,
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
        let (domain, problem) = remove_equalities(domain, problem);

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

        let object_pair_static_information =
            Self::compute_object_pair_static_information(&init_state, &predicates);

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
            object_pair_static_information,
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

    fn compute_object_pair_static_information(
        init_state: &DBState,
        predicates: &[Predicate],
    ) -> HashMap<(usize, usize), HashSet<usize>> {
        let mut object_pair_static_information = HashMap::new();

        for predicate in predicates {
            if predicate.arity != 2 || !predicate.is_static {
                continue;
            }

            for tuple in &init_state.relations[predicate.index].tuples {
                assert_eq!(tuple.len(), 2);
                let object1 = tuple[0];
                let object2 = tuple[1];
                let key = (min(object1, object2), max(object1, object2));
                object_pair_static_information
                    .entry(key)
                    .or_insert_with(HashSet::new)
                    .insert(predicate.index);
            }
        }

        object_pair_static_information
    }

    /// The indices of static predicates that apply to a single object and hence
    /// contribute to the static information of that object.
    pub fn object_static_information_predicates(&self) -> Vec<usize> {
        self.predicates
            .iter()
            .filter(|p| p.is_static && p.arity == 1)
            .map(|p| p.index)
            .sorted()
            .dedup()
            .collect()
    }

    /// The indices of static predicates that apply to a pair of objects and
    /// hence contribute to the static information of that pair.
    pub fn object_pair_static_information_predicates(&self) -> Vec<usize> {
        self.predicates
            .iter()
            .filter(|p| p.is_static && p.arity == 2)
            .map(|p| p.index)
            .sorted()
            .dedup()
            .collect()
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

    pub fn object_pair_static_information(&self) -> &HashMap<(usize, usize), HashSet<usize>> {
        &self.object_pair_static_information
    }

    pub fn satisfy_requirements(&mut self, requirements: &HashSet<Requirement>) {
        if requirements.contains(&Requirement::NoNegativePreconditions) {
            info!("Removing negative preconditions (if any) to satisfy heuristic requirements");
            self.remove_negative_preconditions();
        }
    }

    /// Transform the task by removing negative preconditions from the action
    /// schemas.
    ///
    /// We do this by adding auxiliary predicates that represent the
    /// negation of the negative preconditions, and updating the action schemas
    /// that add or remove the relevant predicate to also remove or add the
    /// auxiliary predicate. We also update the initial state to include atoms
    /// for the auxiliary predicates. Furthermore, we also recompute the static
    /// information for objects and object pairs.
    pub fn remove_negative_preconditions(&mut self) {
        let mut negative_predicates: HashSet<usize> = HashSet::new();
        for action_schema in &self.action_schemas {
            for precondition in action_schema.preconditions() {
                if !precondition.is_negative() {
                    continue;
                }

                let predicate_index = precondition.predicate_index();
                negative_predicates.insert(predicate_index);
            }
        }

        let mut original_to_negative_predicate: HashMap<usize, usize> = HashMap::new();
        for predicate_index in negative_predicates {
            info!(
                "Adding auxiliary negative predicate for {} because it appears as a negative precondition",
                self.predicates[predicate_index].name
            );
            let negative_predicate_index = self.predicates.len();
            self.predicates.push(
                self.predicates[predicate_index]
                    .negative_auxiliary_predicate(negative_predicate_index),
            );
            original_to_negative_predicate.insert(predicate_index, negative_predicate_index);

            if self.nullary_predicates.contains(&predicate_index) {
                self.nullary_predicates.insert(negative_predicate_index);
                self.initial_state
                    .nullary_atoms
                    .push(!self.initial_state.nullary_atoms[predicate_index]);
                // all predicates, even nullary, need to be in the relations
                self.initial_state.relations.push(Relation {
                    predicate_symbol: negative_predicate_index,
                    tuples: BTreeSet::new(),
                });
            } else {
                // This is pretty inefficient -- we need to add all the atoms
                // that don't exist in the initial state for the new predicate
                let existing_tuples: HashSet<SmallTuple> = self.initial_state.relations
                    [predicate_index]
                    .tuples
                    .iter()
                    .cloned()
                    .collect();

                let all_tuples: HashSet<SmallTuple> = self.predicates[predicate_index]
                    .types
                    .iter()
                    .map(|type_index| self.objects_per_type[*type_index].iter())
                    .multi_cartesian_product()
                    .map(|indices| {
                        let raw = indices.iter().copied().copied().collect::<RawSmallTuple>();
                        SmallTuple::new(raw)
                    })
                    .collect();
                let tuples_to_add: HashSet<SmallTuple> =
                    all_tuples.difference(&existing_tuples).cloned().collect();
                self.initial_state.relations.push(Relation {
                    predicate_symbol: negative_predicate_index,
                    tuples: BTreeSet::from_iter(tuples_to_add),
                });
            }
        }

        for action_schema in &mut self.action_schemas {
            action_schema
                .update_with_auxiliary_negative_predicates(&original_to_negative_predicate);
        }

        self.object_static_information = Self::compute_object_static_information(
            &self.initial_state,
            &self.predicates,
            &self.objects,
        );
        self.object_pair_static_information =
            Self::compute_object_pair_static_information(&self.initial_state, &self.predicates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{small_tuple, Atom},
        test_utils::*,
    };

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
    fn objects_per_type_blocksworld() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);

        assert_eq!(task.objects_per_type.len(), 1);
        // object
        assert_eq!(task.objects_per_type[0], HashSet::from([0, 1, 2, 3]));
    }

    #[test]
    fn objects_per_type_spanner() {
        let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);

        assert_eq!(task.objects_per_type.len(), 6);
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
        // we always add a type for objects
        assert_eq!(
            task.objects_per_type[5],
            HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14])
        );
    }

    #[test]
    fn static_predicate_detection_spanner() {
        let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);

        let static_predicates = task.static_predicates();

        // Only the "link" (3) predicate is static in spanner
        assert_eq!(static_predicates, HashSet::from([3]));

        // Even though there are static predicates, this only counts the number
        // with arity 1
        assert_eq!(task.object_static_information_predicates().len(), 0);
    }

    #[test]
    fn static_predicate_detection_childsnack() {
        let task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);

        let static_predicates = task.static_predicates();

        // In childsnack, predicates no_gluten_bread (3), no_gluten_content (4),
        // allergic_gluten(7), not_allrgic_gluten(8), and waiting (10) are
        // static.
        assert_eq!(static_predicates, HashSet::from([3, 4, 7, 8, 10]));

        // waiting doesn't have arity 1, so it doesn't count
        assert_eq!(task.object_static_information_predicates().len(), 4);
    }

    #[test]
    fn object_static_information_childsnack() {
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

    #[test]
    fn object_pair_static_information_satellite() {
        let task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);

        let object_pair_static_information = task.object_pair_static_information();

        // (calibration_target ins1 dir1), (calibration_target ins2 dir3),
        // (on_board ins1 sat1), (on_board ins2 sat2), (supports ins2 mod1),
        // (supports ins1 mod1)
        assert_eq!(
            object_pair_static_information
                .keys()
                .copied()
                .sorted()
                .collect_vec(),
            vec![(0, 2), (1, 3), (2, 4), (2, 5), (3, 4), (3, 7)]
        );

        // on_board ins1 sat1
        assert_eq!(
            object_pair_static_information
                .get(&(0, 2))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![0]
        );

        // on_board ins2 sat2
        assert_eq!(
            object_pair_static_information
                .get(&(1, 3))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![0]
        );

        // supports ins1 mod1
        assert_eq!(
            object_pair_static_information
                .get(&(2, 4))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![1]
        );

        // calibration_target ins1 dir1
        assert_eq!(
            object_pair_static_information
                .get(&(2, 5))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![7]
        );

        // supports ins2 mod1
        assert_eq!(
            object_pair_static_information
                .get(&(3, 4))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![1]
        );

        // calibration_target ins2 dir3
        assert_eq!(
            object_pair_static_information
                .get(&(3, 7))
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<usize>>(),
            vec![7]
        );
    }

    #[test]
    fn negative_precondition_removal_ferry() {
        let mut task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
        task.remove_negative_preconditions();

        // should add an auxiliary predicate for at-ferry
        assert_eq!(task.predicates.len(), 5);
        // should add (not@at-ferry loc2) and (not@at-ferry loc3) to the initial
        // state
        assert_eq!(
            task.initial_state.atoms(),
            vec![
                Atom::new(0, small_tuple![2]),
                Atom::new(1, small_tuple![0, 3]),
                Atom::new(1, small_tuple![1, 4]),
                Atom::new(4, small_tuple![3]),
                Atom::new(4, small_tuple![4]),
                Atom::new(2, small_tuple![])
            ]
        );

        assert_eq!(
            task.action_schemas
                .iter()
                .map(|a| a.to_string())
                .collect_vec(),
            vec![
                "((index 0) \
                (parameters (0 1) (1 1)) \
                (preconditions (0 ?0) (4 ?1)) \
                (effects (0 ?1) (not (0 ?0)) (not (4 ?1)) (4 ?0)))",
                "((index 1) \
                (parameters (0 0) (1 1)) \
                (preconditions (1 ?0 ?1) (0 ?1) (2)) \
                (effects (3 ?0) (not (1 ?0 ?1)) (not (2))))",
                "((index 2) \
                (parameters (0 0) (1 1)) \
                (preconditions (3 ?0) (0 ?1)) \
                (effects (1 ?0 ?1) (2) (not (3 ?0))))"
            ]
        )
    }

    #[test]
    fn negative_precondition_removal_satellite() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();

        // should add an auxiliary predicate for pointing
        assert_eq!(task.predicates.len(), 9);
        // should add (not@pointing sat1 dir1), (not@pointing sat1 dir2),
        // (not@pointing sat2 dir2), (not@pointing sat2 dir3) to the initial
        // state
        assert_eq!(
            task.initial_state.atoms(),
            vec![
                Atom::new(0, small_tuple![2, 0]),
                Atom::new(0, small_tuple![3, 1]),
                Atom::new(1, small_tuple![2, 4]),
                Atom::new(1, small_tuple![3, 4]),
                Atom::new(2, small_tuple![0, 7]),
                Atom::new(2, small_tuple![1, 5]),
                Atom::new(3, small_tuple![0]),
                Atom::new(3, small_tuple![1]),
                Atom::new(7, small_tuple![2, 5]),
                Atom::new(7, small_tuple![3, 7]),
                Atom::new(8, small_tuple![0, 5]),
                Atom::new(8, small_tuple![0, 6]),
                Atom::new(8, small_tuple![1, 6]),
                Atom::new(8, small_tuple![1, 7]),
            ]
        );

        assert_eq!(
            task.action_schemas
                .iter()
                .map(|a| a.to_string())
                .collect_vec(),
            vec![
                "((index 0) \
                (parameters (0 0) (1 1) (2 1)) \
                (preconditions (2 ?0 ?2) (8 ?0 ?1)) \
                (effects (2 ?0 ?1) (not (2 ?0 ?2)) (not (8 ?0 ?1)) (8 ?0 ?2)))",
                "((index 1) \
                (parameters (0 2) (1 0)) \
                (preconditions (0 ?0 ?1) (3 ?1)) \
                (effects (4 ?0) (not (5 ?0)) (not (3 ?1))))",
                "((index 2) \
                (parameters (0 2) (1 0)) \
                (preconditions (0 ?0 ?1) (4 ?0)) \
                (effects (not (4 ?0)) (3 ?1)))",
                "((index 3) \
                (parameters (0 0) (1 2) (2 1)) \
                (preconditions (0 ?1 ?0) (7 ?1 ?2) (2 ?0 ?2) (4 ?1)) \
                (effects (5 ?1)))",
                "((index 4) \
                (parameters (0 0) (1 1) (2 2) (3 3)) \
                (preconditions (5 ?2) (0 ?2 ?0) (1 ?2 ?3) (4 ?2) (2 ?0 ?1)) \
                (effects (6 ?1 ?3)))",
            ]
        )
    }

    #[test]
    fn equalities_are_compiled_away_warehouse() {
        let task = Task::from_text(WAREHOUSE_DOMAIN_TEXT, WAREHOUSE_PROBLEM10_TEXT);

        assert_eq!(task.predicates.len(), 7);
        assert_eq!(task.predicates[0].name, "on");
        assert_eq!(task.predicates[1].name, "on-base");
        assert_eq!(task.predicates[2].name, "clear");
        assert_eq!(task.predicates[3].name, "clear-base");
        assert_eq!(task.predicates[4].name, "to-remove");
        assert_eq!(task.predicates[5].name, "removed");
        assert_eq!(task.predicates[6].name, "@object-equal");

        assert_eq!(task.initial_state.atoms().len(), 56);
    }
}
