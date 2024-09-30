use std::fmt::{Display, Formatter};

use strum_macros::EnumIs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIs, PartialOrd, Ord)]
pub enum Term {
    Object(usize),
    Variable {
        variable_index: usize,
        type_index: usize,
    },
}

impl Term {
    #[inline(always)]
    pub fn new_object(index: usize) -> Self {
        Self::Object(index)
    }

    #[inline(always)]
    pub fn new_variable(variable_index: usize, type_index: usize) -> Self {
        Self::Variable {
            variable_index,
            type_index,
        }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        match self {
            Term::Object(index) => *index,
            Term::Variable {
                variable_index,
                type_index: _,
            } => *variable_index,
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Term::Object(index) => write!(f, "{}", index),
            Term::Variable {
                variable_index,
                type_index: _,
            } => write!(f, "?{}", variable_index),
        }
    }
}
