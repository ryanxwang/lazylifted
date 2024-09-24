use std::fmt::Display;

use crate::search::datalog::{
    atom::Atom, rules::generic_rule::GenericRule, rules::project_rule::ProjectRule,
    rules::rule_core::RuleCore, rules::utils::VariableSource, Annotation,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rule {
    Generic(GenericRule),
    Project(ProjectRule),
}

impl Rule {
    pub fn new_generic(rule: GenericRule) -> Self {
        Self::Generic(rule)
    }

    pub fn new_project(rule: ProjectRule) -> Self {
        Self::Project(rule)
    }

    fn core(&self) -> &RuleCore {
        match self {
            Rule::Generic(rule) => rule.core(),
            Rule::Project(rule) => rule.core(),
        }
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        match self {
            Rule::Generic(rule) => rule.core_mut(),
            Rule::Project(rule) => rule.core_mut(),
        }
    }

    #[inline(always)]
    pub fn effect(&self) -> &Atom {
        self.core().effect()
    }

    #[inline(always)]
    pub fn conditions(&self) -> &[Atom] {
        self.core().conditions()
    }

    /// Update the conditions of the rule. Please make sure that the variable
    /// source is also updated.
    pub fn set_condition(&mut self, conditions: Vec<Atom>) {
        self.core_mut().set_condition(conditions);
    }

    #[inline(always)]
    pub fn weight(&self) -> f64 {
        self.core().weight()
    }

    #[inline(always)]
    pub fn annotation(&self) -> &Annotation {
        self.core().annotation()
    }

    pub fn schema_index(&self) -> Option<usize> {
        match self {
            Rule::Generic(rule) => Some(rule.schema_index()),
            Rule::Project(_) => None,
        }
    }

    #[inline(always)]
    pub fn variable_source(&self) -> &VariableSource {
        self.core().variable_source()
    }

    pub fn variable_source_mut(&mut self) -> &mut VariableSource {
        self.core_mut().variable_source_mut()
    }

    pub fn condition_variables(&self) -> Vec<usize> {
        self.conditions()
            .iter()
            .flat_map(|atom| atom.variables())
            .collect()
    }

    /// Update the condition at the given index, will update the variable source
    /// as well.
    pub fn update_single_condition(&mut self, condition: Atom, index: usize) {
        self.core_mut().update_single_condition(condition, index);
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rule::Generic(rule) => write!(f, "{}", rule),
            Rule::Project(rule) => write!(f, "{}", rule),
        }
    }
}
