use crate::search::{
    datalog::{
        Annotation, AnnotationGenerator, DatalogHeuristicType, DatalogProgram,
        DatalogTransformationOptions, WeightedGrounder, WeightedGrounderConfig,
    },
    DBState, Heuristic, HeuristicValue, Task,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct HaddHeuristic {
    program: DatalogProgram,
    grounder: WeightedGrounder,
}

impl HaddHeuristic {
    pub fn new(task: Rc<Task>) -> Self {
        let program = DatalogProgram::new_with_transformations(
            task,
            &Self::get_annotation_generator(),
            &Self::get_transformation_options(),
        );
        let config = WeightedGrounderConfig {
            heuristic_type: DatalogHeuristicType::Hadd,
        };
        let grounder = WeightedGrounder::new(&program, config);
        Self { program, grounder }
    }

    fn get_annotation_generator() -> AnnotationGenerator {
        Box::new(|_| Annotation::None)
    }

    fn get_transformation_options() -> DatalogTransformationOptions {
        DatalogTransformationOptions::default()
    }
}

impl Heuristic<DBState> for HaddHeuristic {
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

    // Test by checking the hadd value of the initial state of various tasks

    #[test]
    fn test_hadd_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));

        let mut hadd = HaddHeuristic::new(task.clone());
        let h_value = hadd.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(15.0));
    }

    #[test]
    fn test_hadd_childsnack() {
        let mut task = Task::from_text(CHILDSNACK_DOMAIN_TEXT, CHILDSNACK_PROBLEM06_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hadd = HaddHeuristic::new(task.clone());
        let h_value = hadd.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(9.0));
    }

    #[test]
    fn test_hadd_ferry() {
        let mut task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hadd = HaddHeuristic::new(task.clone());
        let h_value = hadd.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(6.0));
    }

    #[test]
    fn test_hadd_spanner() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));

        let mut hadd = HaddHeuristic::new(task.clone());
        let h_value = hadd.evaluate(&task.initial_state, &task);
        // TODO-someday: this value is unnecessarily low (but correct, I think),
        // because we currently ignore type information, meaning that in
        // spanner, nuts and spanners can walk around. Pretty stupid
        assert_eq!(h_value, HeuristicValue::from(6.0));
    }

    #[test]
    fn test_hadd_satellite() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);

        let mut hadd = HaddHeuristic::new(task.clone());
        let h_value = hadd.evaluate(&task.initial_state, &task);
        assert_eq!(h_value, HeuristicValue::from(18.0));
    }
}
