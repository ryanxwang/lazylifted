use crate::search::{
    datalog::{
        Annotation, AnnotationGenerator, DatalogHeuristicType, DatalogProgram,
        DatalogTransformationOptions, RuleCategory, WeightedGrounder, WeightedGrounderConfig,
    },
    successor_generators::SuccessorGeneratorName,
    Action, DBState, Heuristic, HeuristicValue, PartialAction, SuccessorGenerator, Task,
    NO_PARTIAL,
};
use itertools::Itertools;
use ordered_float::Float;
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Debug)]
pub struct FfHeuristic {
    program: DatalogProgram,
    grounder: WeightedGrounder,
    relaxed_plan: Rc<RefCell<HashSet<Action>>>,
    successor_generator: Box<dyn SuccessorGenerator>,
}

impl FfHeuristic {
    pub fn new(
        task: Rc<Task>,
        action_set_mode: bool,
        successor_generator_name: SuccessorGeneratorName,
    ) -> Self {
        let relaxed_plan = Rc::new(RefCell::new(HashSet::new()));
        let program = DatalogProgram::new_with_transformations(
            task.clone(),
            &Self::get_annotation_generator(relaxed_plan.clone()),
            &Self::get_transformation_options(action_set_mode),
        );
        let config = WeightedGrounderConfig {
            heuristic_type: DatalogHeuristicType::Hff,
        };
        let grounder = WeightedGrounder::new(config);
        Self {
            program,
            grounder,
            relaxed_plan,
            successor_generator: successor_generator_name.create(&task),
        }
    }

    fn get_annotation_generator(relaxed_plan: Rc<RefCell<HashSet<Action>>>) -> AnnotationGenerator {
        Box::new(move |rule_category| match rule_category {
            RuleCategory::ActionApplicability { schema_index } => {
                Annotation::ExtractGroundActionAndAddToPlan {
                    plan: relaxed_plan.clone(),
                    schema_index,
                }
            }
            RuleCategory::ActionEffect => Annotation::None,
            RuleCategory::Goal => Annotation::None,
        })
    }

    fn get_transformation_options(action_set_mode: bool) -> DatalogTransformationOptions {
        if action_set_mode {
            DatalogTransformationOptions::default().with_restrict_immediate_applicability()
        } else {
            DatalogTransformationOptions::default()
        }
    }
}

impl Heuristic<DBState> for FfHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        if task.goal.is_satisfied(state) {
            return 0.0.into();
        }
        self.relaxed_plan.borrow_mut().clear();

        let hadd_value = self.grounder.ground(&mut self.program, state, None);
        self.program.cleanup_grounding_data();

        // For deadends, the relaxed plan is going to be empty
        if hadd_value.is_infinite() {
            return HeuristicValue::infinity();
        }

        let hff_value = self.relaxed_plan.borrow().len() as f64;
        hff_value.into()
    }
}

impl Heuristic<(DBState, PartialAction)> for FfHeuristic {
    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        if task.goal.is_satisfied(state) {
            return 0.0.into();
        }
        self.relaxed_plan.borrow_mut().clear();

        let actions = if partial == &NO_PARTIAL {
            task.action_schemas()
                .iter()
                .flat_map(|schema| {
                    self.successor_generator
                        .get_applicable_actions(state, schema)
                })
                .collect_vec()
        } else {
            self.successor_generator
                .get_applicable_actions_from_partial(
                    state,
                    &task.action_schemas()[partial.schema_index()],
                    partial,
                )
        };

        let ground_rules = actions
            .iter()
            .flat_map(|action| {
                self.grounder.converted_ground_action_to_temporary_rules(
                    &self.program,
                    action,
                    Annotation::AddGroundActionToPlan {
                        plan: self.relaxed_plan.clone(),
                        action: action.clone(),
                    },
                    1.0,
                )
            })
            .collect_vec();

        let hadd_value = self
            .grounder
            .ground(&mut self.program, state, Some(ground_rules));
        self.program.cleanup_grounding_data();

        // For deadends, the relaxed plan is going to be empty
        if hadd_value.is_infinite() {
            return HeuristicValue::infinity();
        }

        let hff_value = self.relaxed_plan.borrow().len() as f64;
        hff_value.into()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::{search::successor_generators::SuccessorGeneratorName, test_utils::*};

    // Test by checking the hadd value of the initial state of various tasks

    #[test]
    fn test_hff_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(7.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [3] }",    // pickup b4
                "Action { index: 1, instantiation: [0] }",    // putdown b1
                "Action { index: 2, instantiation: [2, 0] }", // stack b3 b1
                "Action { index: 2, instantiation: [3, 1] }", // stack b4 b2
                "Action { index: 3, instantiation: [0, 1] }", // unstack b1 b2
                "Action { index: 3, instantiation: [1, 2] }", // unstack b2 b3
                "Action { index: 3, instantiation: [2, 3] }", // unstack b3 b4
            ]
        );
    }

    #[test]
    fn test_hff_childsnack_p06() {
        let mut task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(6.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [4, 6, 8] }", // make_sandwich_no_gluten sandw2 bread2 content2
                "Action { index: 1, instantiation: [4, 6, 8] }", // make_sandwich sandw2 bread2 content2
                "Action { index: 2, instantiation: [4, 2] }",    // put_on_tray sandw2 tray1
                "Action { index: 3, instantiation: [4, 0, 2, 9] }", // serve_sandwich_no_gluten sandw2 child1 tray1 table1
                "Action { index: 4, instantiation: [4, 1, 2, 9] }", // serve_sandwich sandw2 child2 tray1 table1
                "Action { index: 5, instantiation: [2, 11, 9] }", // move_tray tray1 kitchen table1
            ]
        );
    }

    #[test]
    fn test_hff_childsnack_deadend_p10() {
        let mut task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let successor_generator = SuccessorGeneratorName::FullReducer.create(&task);
        let mut state = task.initial_state.clone();

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::from(12.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [10, 18, 21] }", // make_sandwich_no_gluten sandw4 bread5 content2
                "Action { index: 1, instantiation: [10, 19, 20] }", // make_sandwich sandw4 bread6 content1
                "Action { index: 2, instantiation: [10, 6] }",      // put_on_tray sandw4 tray1
                "Action { index: 3, instantiation: [10, 0, 6, 26] }", // serve_sandwich_no_gluten sandw4 child1 tray1 table1
                "Action { index: 3, instantiation: [10, 1, 6, 28] }", // serve_sandwich_no_gluten sandw4 child2 tray1 table3
                "Action { index: 4, instantiation: [10, 2, 6, 26] }", // serve_sandwich sandw4 child3 tray1 table1
                "Action { index: 4, instantiation: [10, 3, 6, 26] }", // serve_sandwich sandw4 child4 tray1 table1
                "Action { index: 4, instantiation: [10, 4, 6, 28] }", // serve_sandwich sandw4 child5 tray1 table3
                "Action { index: 4, instantiation: [10, 5, 6, 27] }", // serve_sandwich sandw4 child6 tray1 table2
                "Action { index: 5, instantiation: [6, 29, 26] }",    // move tray1 kitchen table1
                "Action { index: 5, instantiation: [6, 29, 27] }",    // move tray1 kitchen table2
                "Action { index: 5, instantiation: [6, 29, 28] }",    // move tray1 kitchen table3
            ]
        );

        // execute make_sandwich_no_gluten sandw1 bread4 content1
        let action = Action::new(0, vec![7, 18, 20]);
        state = successor_generator.generate_successor(&state, &task.action_schemas()[0], &action);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::from(10.0));

        // execute make_sandwich sandw2 bread3 content5
        let action = Action::new(1, vec![8, 17, 24]);
        state = successor_generator.generate_successor(&state, &task.action_schemas()[1], &action);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::from(11.0));

        // execute put_on_tray sandw1 tray1
        let action = Action::new(2, vec![7, 6]);
        state = successor_generator.generate_successor(&state, &task.action_schemas()[2], &action);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::from(9.0));

        // execute move_tray tray1 kitchen table2
        let action = Action::new(5, vec![6, 29, 27]);
        state = successor_generator.generate_successor(&state, &task.action_schemas()[5], &action);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::from(8.0));

        // execute serve_sandwich sandw1 child6 tray1 table2
        let action = Action::new(4, vec![7, 5, 6, 27]);
        state = successor_generator.generate_successor(&state, &task.action_schemas()[4], &action);
        let h_value = hff.evaluate(&state, &task);
        assert_eq!(h_value, HeuristicValue::infinity());
    }

    #[test]
    fn test_hff_ferry() {
        let mut task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(6.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [2, 3] }", // sail loc1 loc2
                "Action { index: 0, instantiation: [2, 4] }", // sail loc1 loc3
                "Action { index: 1, instantiation: [0, 3] }", // board car1 loc2
                "Action { index: 1, instantiation: [1, 4] }", // board car2 loc3
                "Action { index: 2, instantiation: [0, 2] }", // debark car1 loc1
                "Action { index: 2, instantiation: [1, 2] }", // debark car2 loc1
            ]
        );
    }

    #[test]
    fn test_hff_spanner_p01() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM01_TEXT));

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(7.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [3, 4, 0] }", // walk shed location1 bob
                "Action { index: 0, instantiation: [4, 5, 0] }", // walk location1 location2 bob
                "Action { index: 0, instantiation: [5, 6, 0] }", // walk location2 location3 bob
                "Action { index: 0, instantiation: [6, 7, 0] }", // walk location3 location4 bob
                "Action { index: 0, instantiation: [7, 8, 0] }", // walk location4 gate bob
                "Action { index: 1, instantiation: [4, 1, 0] }", // pickup_spanner location1 spanner1 bob
                "Action { index: 2, instantiation: [8, 1, 0, 2] }", // tighten_nut gate spanner1 bob nut1
            ]
        );
    }

    #[test]
    fn test_hff_spanner_p10() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(10.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [10, 11, 0] }", // walk location3 location4 bob
                "Action { index: 0, instantiation: [11, 12, 0] }", // walk location4 location5 bob
                "Action { index: 0, instantiation: [12, 13, 0] }", // walk location5 location6 bob
                "Action { index: 0, instantiation: [13, 14, 0] }", // walk location6 gate bob
                "Action { index: 0, instantiation: [7, 8, 0] }",   // walk shed location1 bob
                "Action { index: 0, instantiation: [8, 9, 0] }",   // walk location1 location2 bob
                "Action { index: 0, instantiation: [9, 10, 0] }",  // walk location2 location3 bob
                "Action { index: 1, instantiation: [10, 1, 0] }", // pickup_spanner location3 spanner1 bob
                "Action { index: 2, instantiation: [14, 1, 0, 5] }", // tighten_nut gate spanner1 bob nut1
                "Action { index: 2, instantiation: [14, 1, 0, 6] }", // tighten_nut gate spanner1 bob nut2
            ]
        );
    }

    #[test]
    fn test_hff_satellite() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(10.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [0, 5, 7] }", // turn_to sat1 dir1 dir3
                "Action { index: 0, instantiation: [0, 6, 7] }", // turn_to sat1 dir2 dir3
                "Action { index: 0, instantiation: [1, 7, 5] }", // turn_to sat2 dir3 dir1
                "Action { index: 1, instantiation: [2, 0] }",    // switch_on ins1 sat1
                "Action { index: 1, instantiation: [3, 1] }",    // switch_on ins2 sat2
                "Action { index: 3, instantiation: [0, 2, 5] }", // calibrate sat1 ins1 dir1
                "Action { index: 3, instantiation: [1, 3, 7] }", // calibrate sat2 ins2 dir3
                "Action { index: 4, instantiation: [0, 6, 2, 4] }", // take_image sat1 dir2 ins1 mod1
                "Action { index: 4, instantiation: [0, 7, 2, 4] }", // take_image sat1 dir3 ins1 mod1
                "Action { index: 4, instantiation: [1, 5, 3, 4] }", // take_image sat2 dir1 ins2 mod1
            ]
        );
    }

    #[test]
    fn test_hff_transport() {
        let mut task = Task::from_text(TRANSPORT_DOMAIN_TEXT, TRANSPORT_PROBLEM16_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone(), false, SuccessorGeneratorName::FullReducer);
        let h_value = hff.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(28.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [0, 17, 16] }", // drive v1 l5 l4
                "Action { index: 0, instantiation: [0, 17, 21] }", // drive v1 l5 l9
                "Action { index: 0, instantiation: [0, 21, 13] }", // drive v1 l5 l1
                "Action { index: 0, instantiation: [1, 15, 20] }", // drive v2 l3 l8
                "Action { index: 0, instantiation: [1, 19, 15] }", // drive v2 l7 l3
                "Action { index: 0, instantiation: [3, 14, 13] }", // drive v2 l2 l1
                "Action { index: 0, instantiation: [3, 14, 19] }", // drive v2 l2 l7
                "Action { index: 0, instantiation: [3, 14, 21] }", // drive v2 l2 l9
                "Action { index: 0, instantiation: [3, 14, 22] }", // drive v2 l2 l10
                "Action { index: 0, instantiation: [3, 15, 20] }", // drive v2 l3 l8
                "Action { index: 0, instantiation: [3, 19, 15] }", // drive v2 l7 l3
                "Action { index: 0, instantiation: [3, 22, 18] }", // drive v2 l10 l6
                "Action { index: 1, instantiation: [0, 13, 10, 23, 24] }", // pick-up v1 l1 p6 c0 c1
                "Action { index: 1, instantiation: [0, 16, 8, 23, 24] }", // pick-up v1 l4 p4 c0 c1
                "Action { index: 1, instantiation: [0, 17, 11, 23, 24] }", // pick-up v1 l5 p7 c0 c1
                "Action { index: 1, instantiation: [1, 19, 9, 24, 25] }", // pick-up v2 l7 p5 c1 c2
                "Action { index: 1, instantiation: [3, 14, 6, 23, 24] }", // pick-up v4 l2 p2 c0 c1
                "Action { index: 1, instantiation: [3, 18, 7, 23, 24] }", // pick-up v4 l6 p3 c0 c1
                "Action { index: 1, instantiation: [3, 20, 12, 23, 24] }", // pick-up v4 l8 p8 c0 c1
                "Action { index: 1, instantiation: [3, 21, 5, 23, 24] }", // pick-up v4 l9 p1 c0 c1
                "Action { index: 2, instantiation: [0, 13, 8, 24, 25] }", // drop v1 l1 p4 c1 c2
                "Action { index: 2, instantiation: [0, 16, 11, 24, 25] }", // drop v1 l4 p7 c1 c2
                "Action { index: 2, instantiation: [0, 17, 10, 24, 25] }", // drop v1 l5 p6 c1 c2
                "Action { index: 2, instantiation: [1, 20, 9, 24, 25] }", // drop v1 l8 p5 c1 c2
                "Action { index: 2, instantiation: [3, 13, 12, 24, 25] }", // drop v1 l1 p8 c1 c2
                "Action { index: 2, instantiation: [3, 13, 6, 24, 25] }", // drop v1 l1 p2 c1 c2
                "Action { index: 2, instantiation: [3, 13, 7, 24, 25] }", // drop v1 l1 p3 c1 c2
                "Action { index: 2, instantiation: [3, 20, 5, 24, 25] }", // drop v1 l8 p1 c1 c2
            ]
        );
    }

    #[test]
    fn test_hff_partial_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));

        let mut hff = FfHeuristic::new(task.clone(), true, SuccessorGeneratorName::FullReducer);

        // Test with NO_PARTIAL
        let h_value = hff.evaluate(&(task.initial_state.clone(), NO_PARTIAL), &task);
        assert_eq!(h_value, HeuristicValue::from(7.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [3] }",    // pickup b4
                "Action { index: 1, instantiation: [0] }",    // putdown b1
                "Action { index: 2, instantiation: [2, 0] }", // stack b3 b1
                "Action { index: 2, instantiation: [3, 1] }", // stack b4 b2
                "Action { index: 3, instantiation: [0, 1] }", // unstack b1 b2
                "Action { index: 3, instantiation: [1, 2] }", // unstack b2 b3
                "Action { index: 3, instantiation: [2, 3] }", // unstack b3 b4
            ]
        );

        // Test with fixed action schema pickup, which is inapplicable
        let h_value = hff.evaluate(
            &(task.initial_state.clone(), PartialAction::new(0, vec![])),
            &task,
        );
        assert_eq!(h_value, HeuristicValue::infinity());
    }
}
