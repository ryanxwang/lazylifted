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
            Self::get_annotation_generator(),
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
    fn evaluate(&mut self, _state: &DBState, _task: &Task) -> HeuristicValue {
        todo!("Implement HmaxHeuristic::evaluate")
    }
}
