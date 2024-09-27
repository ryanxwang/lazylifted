#![allow(dead_code)]

use crate::search::{
    datalog::{
        Annotation, AnnotationGenerator, DatalogHeuristicType, DatalogProgram,
        DatalogTransformationOptions, WeightedGrounder, WeightedGrounderConfig,
    },
    DBState, Heuristic, HeuristicValue, Task,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct HmaxHeuristic {
    program: DatalogProgram,
    grounder: WeightedGrounder,
}

impl HmaxHeuristic {
    pub fn new(task: Rc<Task>) -> Self {
        let program = DatalogProgram::new_with_transformations(
            task,
            &Self::get_annotation_generator(),
            &Self::get_transformation_options(),
        );
        let config = WeightedGrounderConfig {
            heuristic_type: DatalogHeuristicType::Hmax,
        };
        let grounder = WeightedGrounder::new(&program, config);
        Self { program, grounder }
    }

    fn get_annotation_generator() -> AnnotationGenerator {
        Box::new(|_head, _task| Annotation::None)
    }

    fn get_transformation_options() -> DatalogTransformationOptions {
        DatalogTransformationOptions::default()
    }
}

impl Heuristic<DBState> for HmaxHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        if task.goal.is_satisfied(state) {
            return 0.0.into();
        }

        let h = self.grounder.ground(&mut self.program, state);
        self.program.cleanup_grounding_data();
        h.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    // Test by checking the hmax value of the initial state of various tasks

    #[test]
    fn test_hmax_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));

        let mut hmax = HmaxHeuristic::new(task.clone());
        let h_value = hmax.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(5.0));
    }

    #[test]
    fn test_hmax_childsnack() {
        let mut task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hmax = HmaxHeuristic::new(task.clone());
        let h_value = hmax.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(3.0));
    }

    #[test]
    fn test_hmax_ferry() {
        let mut task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hmax = HmaxHeuristic::new(task.clone());
        let h_value = hmax.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(3.0));
    }

    #[test]
    fn test_hmax_spanner() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));

        let mut hmax = HmaxHeuristic::new(task.clone());
        let h_value = hmax.evaluate(&task.initial_state, &task);
        // TODO-someday: this value is unnecessarily low (but correct, I think),
        // because we currently ignore type information, meaning that in
        // spanner, nuts and spanners can walk around. Pretty stupid
        assert_eq!(h_value, HeuristicValue::from(4.0));
    }

    #[test]
    fn test_hmax_satellite() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hmax = HmaxHeuristic::new(task.clone());
        let h_value = hmax.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(3.0));
    }
}
