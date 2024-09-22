use std::rc::Rc;

use crate::search::{
    datalog::{fact::Fact, transformation_options::TransformationOptions, AnnotationGenerator},
    Task,
};

#[derive(Debug)]
pub struct Program {
    #[allow(dead_code)]
    facts: Vec<Fact>,
    #[allow(dead_code)]
    task: Rc<Task>,
}

impl Program {
    pub fn new_with_transformations(
        _task: Rc<Task>,
        _annotation_generator: AnnotationGenerator,
        _transformation_options: &TransformationOptions,
    ) -> Self {
        todo!("Implement Program::new_with_transformations");
    }
}
