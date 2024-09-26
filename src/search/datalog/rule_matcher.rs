use std::collections::HashMap;

use crate::search::datalog::rules::{Rule, RuleIndex};

/// A [`Match`] is a reference by indices to a rule and a condition in the rule.
#[derive(Debug, Clone)]
pub struct Match {
    pub rule_index: RuleIndex,
    pub condition_index: usize,
}

#[derive(Debug, Clone)]
pub struct RuleMatcher {
    /// A map from predicate indices to everywhere that the predicate appears in
    /// the rules.
    matches: HashMap<usize, Vec<Match>>,
}

impl RuleMatcher {
    pub fn new(rules: &[Rule]) -> Self {
        let mut matches: HashMap<usize, Vec<Match>> = HashMap::new();
        for rule in rules {
            for (condition_index, condition) in rule.conditions().iter().enumerate() {
                matches
                    .entry(condition.predicate_index())
                    .or_default()
                    .push(Match {
                        rule_index: rule.index(),
                        condition_index,
                    });
            }
        }

        Self { matches }
    }

    pub fn get_matched_rules(&self, predicate_index: usize) -> &[Match] {
        self.matches
            .get(&predicate_index)
            .map_or(&[], |matches| matches.as_slice())
    }
}
