use std::{collections::HashMap, fmt::Display};

use crate::search::datalog::{
    annotation, atom::Atom, rules::rule_core::RuleCore, term::Term, Annotation,
};

#[derive(Debug, Clone, PartialEq)]
pub struct JoinRule {
    core: RuleCore,
    joining_variable_positions: HashMap<Term, (usize, usize)>,
}

impl JoinRule {
    pub fn new(
        effect: Atom,
        conditions: (Atom, Atom),
        weight: f64,
        annotation: Annotation,
    ) -> Self {
        let core = RuleCore::new(effect, vec![conditions.0, conditions.1], weight, annotation);
        Self::new_from_core(core)
    }

    pub(super) fn new_from_core(core: RuleCore) -> Self {
        assert_eq!(core.conditions().len(), 2);
        let mut joining_variable_positions = HashMap::new();
        for (i, term1) in core.conditions()[0].arguments().iter().enumerate() {
            if term1.is_object() {
                continue;
            }
            for (j, term2) in core.conditions()[1].arguments().iter().enumerate() {
                if term1 == term2 {
                    joining_variable_positions.insert(term1.to_owned(), (i, j));
                }
            }
        }

        Self {
            core,
            joining_variable_positions,
        }
    }

    pub fn core(&self) -> &RuleCore {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }
}

impl Display for JoinRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}
