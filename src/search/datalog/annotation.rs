use std::{fmt::Display, rc::Rc};

use crate::search::{datalog::program::Program, Task};

/// The category of a rule, which tells the annotation generator what annotation
/// it should generate
#[derive(Debug, Clone, Copy)]
pub enum RuleCategory {
    ActionApplicability { schema_index: usize },
    ActionEffect,
    Goal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Annotation::None => write!(f, "None"),
        }
    }
}

pub type AnnotationGenerator = Box<dyn Fn(RuleCategory, Rc<Task>) -> Annotation>;
