use crate::search::{
    datalog::{
        Annotation, AnnotationGenerator, DatalogHeuristicType, DatalogProgram,
        DatalogTransformationOptions, RuleCategory, WeightedGrounder, WeightedGrounderConfig,
    },
    Action, DBState, Heuristic, HeuristicValue, Task,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Debug)]
pub struct FfHeuristic {
    program: DatalogProgram,
    grounder: WeightedGrounder,
    relaxed_plan: Rc<RefCell<HashSet<Action>>>,
}

impl FfHeuristic {
    pub fn new(task: Rc<Task>) -> Self {
        let relaxed_plan = Rc::new(RefCell::new(HashSet::new()));
        let program = DatalogProgram::new_with_transformations(
            task,
            &Self::get_annotation_generator(relaxed_plan.clone()),
            &Self::get_transformation_options(),
        );
        let config = WeightedGrounderConfig {
            heuristic_type: DatalogHeuristicType::Hff,
        };
        let grounder = WeightedGrounder::new(&program, config);
        Self {
            program,
            grounder,
            relaxed_plan,
        }
    }

    fn get_annotation_generator(relaxed_plan: Rc<RefCell<HashSet<Action>>>) -> AnnotationGenerator {
        Box::new(move |rule_category| match rule_category {
            RuleCategory::ActionApplicability { schema_index } => Annotation::AddToRelaxedPlan {
                plan: relaxed_plan.clone(),
                schema_index,
            },
            RuleCategory::ActionEffect => Annotation::None,
            RuleCategory::Goal => Annotation::None,
        })
    }

    fn get_transformation_options() -> DatalogTransformationOptions {
        DatalogTransformationOptions::default()
    }
}

impl Heuristic<DBState> for FfHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        if task.goal.is_satisfied(state) {
            return 0.0.into();
        }
        self.relaxed_plan.borrow_mut().clear();

        let _hadd_value = self.grounder.ground(&mut self.program, state);
        self.program.cleanup_grounding_data();

        let hff_value = self.relaxed_plan.borrow().len() as f64;
        hff_value.into()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::test_utils::*;

    // Test by checking the hadd value of the initial state of various tasks

    #[test]
    fn test_hadd_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));

        let mut hff = FfHeuristic::new(task.clone());
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
    fn test_hadd_childsnack() {
        let mut task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone());
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
                "Action { index: 0, instantiation: [3, 6, 8] }", // make_sandwich_no_gluten sandw1 bread2 content2
                "Action { index: 1, instantiation: [3, 6, 8] }", // make_sandwich sandw1 bread2 content2
                "Action { index: 2, instantiation: [3, 2] }",    // put_on_tray sandw1 tray1
                "Action { index: 3, instantiation: [3, 0, 2, 9] }", // serve_sandwich_no_gluten sandw1 child1 tray1 table1
                "Action { index: 4, instantiation: [3, 1, 2, 9] }", // serve_sandwich sandw1 child2 tray1 table1
                "Action { index: 5, instantiation: [2, 11, 9] }", // move_tray tray1 kitchen table1
            ]
        );
    }

    #[test]
    fn test_hadd_ferry() {
        let mut task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone());
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
    fn test_hadd_spanner() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));

        let mut hff = FfHeuristic::new(task.clone());
        let h_value = hff.evaluate(&task.initial_state, &task);
        // TODO-someday: this value is unnecessarily low (but correct, I think),
        // because we currently ignore type information, meaning that in
        // spanner, nuts and spanners can walk around. Pretty stupid
        assert_eq!(h_value, HeuristicValue::from(4.0));
        assert_eq!(
            hff.relaxed_plan
                .borrow()
                .iter()
                .map(|action| { format!("{:?}", action) })
                .sorted()
                .collect_vec(),
            vec![
                "Action { index: 0, instantiation: [13, 14, 2] }", // walk location6 gate spanner2
                "Action { index: 1, instantiation: [13, 2, 2] }", // pickup_spanner location6 spanner2 spanner2
                "Action { index: 2, instantiation: [14, 2, 2, 5] }", // tighten_nut gate spanner2 spanner2 nut1
                "Action { index: 2, instantiation: [14, 2, 2, 6] }", // tighten_nut gate spanner2 spanner2 nut2
            ]
        );
    }

    #[test]
    fn test_hadd_satellite() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hff = FfHeuristic::new(task.clone());
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
}
