use std::fmt::Display;

use crate::search::datalog::{atom::Atom, rules::rule_core::RuleCore, Annotation};

/// A [`GenericRule`] that corresponds to a rule generated from an action
/// schema. It is a simple wrapper around a [`RuleCore`] with an additional
/// field to store the index of the schema that generated the rule.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GenericRule {
    core: RuleCore,
    schema_index: usize,
}

impl GenericRule {
    pub fn new(
        effect: Atom,
        conditions: Vec<Atom>,
        weight: f64,
        annotation: Annotation,
        schema_index: usize,
    ) -> Self {
        let core = RuleCore::new(effect, conditions, weight, annotation);
        Self { core, schema_index }
    }

    #[inline(always)]
    pub fn core(&self) -> &RuleCore {
        &self.core
    }

    #[inline(always)]
    pub fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }

    #[inline(always)]
    pub fn schema_index(&self) -> usize {
        self.schema_index
    }
}

impl Display for GenericRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}; schema_index: {})", self.core, self.schema_index)
    }
}
