use std::fmt::{Display, Formatter};

use strum_macros::EnumIs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermType {
    Object,
    Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIs)]
pub enum Term {
    Object(usize),
    Variable(usize),
}

impl Term {
    #[inline(always)]
    pub fn new_object(index: usize) -> Self {
        Self::Object(index)
    }

    #[inline(always)]
    pub fn new_variable(index: usize) -> Self {
        Self::Variable(index)
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        match self {
            Term::Object(index) => *index,
            Term::Variable(index) => *index,
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Term::Object(index) => write!(f, "{}", index),
            Term::Variable(index) => write!(f, "?{}", index),
        }
    }
}
