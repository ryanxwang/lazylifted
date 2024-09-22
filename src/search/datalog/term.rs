use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermType {
    Object,
    Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Term {
    pub index: usize,
    pub term_type: TermType,
}

impl Term {
    pub fn new(index: usize, term_type: TermType) -> Self {
        Self { index, term_type }
    }

    pub fn new_object(index: usize) -> Self {
        Self::new(index, TermType::Object)
    }

    pub fn new_variable(index: usize) -> Self {
        Self::new(index, TermType::Variable)
    }

    pub fn is_object(&self) -> bool {
        self.term_type == TermType::Object
    }

    pub fn is_variable(&self) -> bool {
        self.term_type == TermType::Variable
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.term_type {
            TermType::Object => write!(f, "{}", self.index),
            TermType::Variable => write!(f, "?{}", self.index),
        }
    }
}
