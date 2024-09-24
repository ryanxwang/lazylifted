use std::fmt::Display;

use crate::search::datalog::{
    arguments::Arguments, atom::Atom, rules::rule_core::RuleCore, Annotation,
};

#[derive(Debug, Clone, Default)]
struct ReachedFacts {
    facts: Vec<Arguments>,
    fact_indices: Vec<usize>,
    costs: Vec<f64>,
}

impl PartialEq for ReachedFacts {
    fn eq(&self, other: &Self) -> bool {
        self.facts == other.facts
            && self.fact_indices == other.fact_indices
            && self.costs == other.costs
    }
}

/// A [`ProductRule`] is a rule with multiple conditions, but none of them share
/// any variables.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductRule {
    core: RuleCore,
    reached_facts_per_condition: Vec<ReachedFacts>,
}

impl ProductRule {
    pub fn new(effect: Atom, conditions: Vec<Atom>, weight: f64, annotation: Annotation) -> Self {
        let core = RuleCore::new(effect, conditions, weight, annotation);
        Self::new_from_core(core)
    }

    pub(super) fn new_from_core(core: RuleCore) -> Self {
        // We keep these asserts for safety because this should only ever run
        // during preprocessing.
        assert!(core.conditions().len() > 1);
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

        let reached_facts_per_condition = core
            .conditions()
            .iter()
            .map(|_| ReachedFacts::default())
            .collect();
        Self {
            core,
            reached_facts_per_condition,
        }
    }

    pub fn core(&self) -> &RuleCore {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }
}

impl Display for ProductRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}
