use std::fmt::Display;

use crate::search::datalog::{
    atom::Atom,
    rules::{rule_core::RuleCore, RuleTrait},
    Annotation,
};

/// A [`ProjectRule`] is a special rule that is used to project an atom to
/// another atom. This means that it is a wrapper around [`RuleCore`] where the
/// condition has exactly one atom, and all the variables in the effect also
/// appear in the condition. See the following paper for more,
///
/// Helmert, M. 2009. Concise Finite-Domain Pepresentations for PDDL Planning
/// Tasks. AIJ, 173: 503-535.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectRule {
    core: RuleCore,
}

impl ProjectRule {
    /// Create a new [`ProjectRule`] with the given effect, condition, weight,
    /// and annotation.
    pub fn new(effect: Atom, condition: Atom, weight: f64, annotation: Annotation) -> Self {
        let core = RuleCore::new(effect, vec![condition], weight, annotation);
        Self::new_from_core(core)
    }

    pub(super) fn new_from_core(core: RuleCore) -> Self {
        assert!(core.effect().is_variable_unique());
        assert_eq!(core.conditions().len(), 1);
        assert!(core.conditions()[0].is_variable_unique());
        assert!(core.conditions()[0]
            .variables_set()
            .is_superset(&core.effect().variables_set()));

        Self { core }
    }
}

impl Display for ProjectRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}

impl RuleTrait for ProjectRule {
    fn core(&self) -> &RuleCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }
}
