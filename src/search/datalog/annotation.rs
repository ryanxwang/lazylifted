use std::rc::Rc;

use crate::search::{datalog::program::Program, Task};

#[derive(Debug)]
pub enum Annotation {
    None,
}

impl Annotation {
    pub fn execute(&self, _head: usize, _program: &Program) {
        match self {
            Annotation::None => {}
        }
    }
}

pub type AnnotationGenerator = Box<dyn Fn(usize, Rc<Task>) -> Annotation>;
