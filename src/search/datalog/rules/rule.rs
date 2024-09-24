use std::fmt::Display;

use crate::search::datalog::{
    atom::Atom, rules::generic_rule::GenericRule, rules::project_rule::ProjectRule,
    rules::utils::VariableSource, Annotation,
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

    pub fn effect(&self) -> &Atom {
        match self {
            Rule::Generic(rule) => rule.core().effect(),
            Rule::Project(rule) => rule.core().effect(),
        }
    }

    pub fn conditions(&self) -> &[Atom] {
        match self {
            Rule::Generic(rule) => rule.core().conditions(),
            Rule::Project(rule) => rule.core().conditions(),
        }
    }

    /// Update the conditions of the rule. Please make sure that the variable
    /// source is also updated.
    pub fn set_condition(&mut self, conditions: Vec<Atom>) {
        match self {
            Rule::Generic(rule) => rule.core_mut().set_condition(conditions),
            Rule::Project(rule) => rule.core_mut().set_condition(conditions),
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Rule::Generic(rule) => rule.core().weight(),
            Rule::Project(rule) => rule.core().weight(),
        }
    }

    pub fn annotation(&self) -> &Annotation {
        match self {
            Rule::Generic(rule) => rule.core().annotation(),
            Rule::Project(rule) => rule.core().annotation(),
        }
    }

    pub fn schema_index(&self) -> Option<usize> {
        match self {
            Rule::Generic(rule) => Some(rule.schema_index()),
            Rule::Project(_) => None,
        }
    }

    pub fn variable_source(&self) -> &VariableSource {
        match self {
            Rule::Generic(rule) => rule.core().variable_source(),
            Rule::Project(rule) => rule.core().variable_source(),
        }
    }

    pub fn variable_source_mut(&mut self) -> &mut VariableSource {
        match self {
            Rule::Generic(rule) => rule.core_mut().variable_source_mut(),
            Rule::Project(rule) => rule.core_mut().variable_source_mut(),
        }
    }

    pub fn condition_variables(&self) -> Vec<usize> {
        self.conditions()
            .iter()
            .flat_map(|atom| atom.variables())
            .collect()
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
