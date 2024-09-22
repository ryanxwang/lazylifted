use crate::search::datalog::rules::generic_rule::GenericRule;

#[derive(Debug, Clone)]
pub enum Rule {
    Generic(GenericRule),
}

impl Rule {
    pub fn new_generic(rule: GenericRule) -> Self {
        Self::Generic(rule)
    }
}
