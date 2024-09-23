use std::fmt::Display;

use crate::search::datalog::{atom::Atom, rules::generic_rule::GenericRule, Annotation};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rule {
    Generic(GenericRule),
}

impl Rule {
    pub fn new_generic(rule: GenericRule) -> Self {
        Self::Generic(rule)
    }

    pub fn effect(&self) -> &Atom {
        match self {
            Rule::Generic(rule) => rule.core().effect(),
        }
    }

    pub fn conditions(&self) -> &[Atom] {
        match self {
            Rule::Generic(rule) => rule.core().conditions(),
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Rule::Generic(rule) => rule.core().weight(),
        }
    }

    pub fn annotation(&self) -> &Annotation {
        match self {
            Rule::Generic(rule) => rule.core().annotation(),
        }
    }

    pub fn schema_index(&self) -> Option<usize> {
        match self {
            Rule::Generic(rule) => Some(rule.schema_index()),
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rule::Generic(rule) => write!(f, "{}", rule),
        }
    }
}
