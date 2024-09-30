use std::{collections::HashSet, fmt::Display};

use crate::search::datalog::{
    atom::Atom, rules::rule_core::RuleCore, rules::RuleTrait, Annotation,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TemporaryGroundRule {
    core: RuleCore,
    unachieved_preconditions: HashSet<Atom>,
}

impl TemporaryGroundRule {
    pub fn new(effect: Atom, conditions: Vec<Atom>, weight: f64, annotation: Annotation) -> Self {
        let core = RuleCore::new(effect, conditions, weight, annotation);
        Self {
            unachieved_preconditions: Self::default_unachievable_preconditions(&core),
            core,
        }
    }

    fn default_unachievable_preconditions(core: &RuleCore) -> HashSet<Atom> {
        core.conditions().iter().cloned().collect()
    }

    pub fn register_reached_fact(&mut self, fact: &Atom) {
        self.unachieved_preconditions.remove(fact);
    }

    pub fn all_preconditions_reached(&self) -> bool {
        self.unachieved_preconditions.is_empty()
    }
}

impl Display for TemporaryGroundRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}

impl RuleTrait for TemporaryGroundRule {
    fn core(&self) -> &RuleCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }

    fn cleanup_grounding_data(&mut self) {
        self.unachieved_preconditions = Self::default_unachievable_preconditions(&self.core);
    }
}
