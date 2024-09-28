use std::fmt::Display;

use crate::search::datalog::{fact::FactId, rules::RuleIndex};

#[derive(Debug, Clone)]
pub struct Achiever {
    rule_index: RuleIndex,
    rule_body: Vec<FactId>,
}

impl Achiever {
    pub fn new(rule_index: RuleIndex, rule_body: Vec<FactId>) -> Self {
        Self {
            rule_index,
            rule_body,
        }
    }

    pub fn rule_index(&self) -> RuleIndex {
        self.rule_index
    }

    pub fn rule_body(&self) -> &[FactId] {
        &self.rule_body
    }
}

impl Display for Achiever {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(rule_index: {}, rule_body: {:?})",
            self.rule_index, self.rule_body
        )
    }
}
