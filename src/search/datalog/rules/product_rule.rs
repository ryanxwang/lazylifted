use std::fmt::Display;

use crate::search::datalog::{
    atom::Atom,
    fact::Fact,
    rules::{rule_core::RuleCore, RuleTrait},
    Annotation,
};

/// A [`ProductRule`] is a rule with multiple conditions, but none of them share
/// any variables.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductRule {
    core: RuleCore,
    /// The reached facts for each condition. For each condition, the facts are
    /// stored in the order they were reached, which should be cheapest to most
    /// expensive by nature of grounding.
    reached_facts_per_condition: Vec<Vec<Fact>>,
}

impl ProductRule {
    pub fn new(effect: Atom, conditions: Vec<Atom>, weight: f64, annotation: Annotation) -> Self {
        let core = RuleCore::new(effect, conditions, weight, annotation);
        Self::new_from_core(core)
    }

    pub(super) fn new_from_core(core: RuleCore) -> Self {
        for i in 0..core.conditions().len() {
            for j in (i + 1)..core.conditions().len() {
                assert!(
                    core.conditions()[i]
                        .variables_set()
                        .is_disjoint(&core.conditions()[j].variables_set()),
                    "Conditions must not share variables"
                );
            }
        }

        let reached_facts_per_condition = core.conditions().iter().map(|_| vec![]).collect();
        Self {
            core,
            reached_facts_per_condition,
        }
    }

    pub fn add_reached_fact(&mut self, condition_index: usize, fact: Fact) {
        self.reached_facts_per_condition[condition_index].push(fact);
    }

    pub fn reached_facts(&self, condition_index: usize) -> &[Fact] {
        &self.reached_facts_per_condition[condition_index]
    }
}

impl Display for ProductRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}

impl RuleTrait for ProductRule {
    fn core(&self) -> &RuleCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }

    fn cleanup_grounding_data(&mut self) {
        for facts in &mut self.reached_facts_per_condition {
            facts.clear();
        }
    }
}
