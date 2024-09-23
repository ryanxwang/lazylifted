use std::fmt::Display;

use crate::search::datalog::rules::generic_rule::GenericRule;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rule {
    Generic(GenericRule),
}

impl Rule {
    pub fn new_generic(rule: GenericRule) -> Self {
        Self::Generic(rule)
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rule::Generic(rule) => write!(f, "{}", rule),
        }
    }
}
