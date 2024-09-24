use std::fmt::Display;

use strum_macros::EnumIs;

use crate::search::datalog::{
    atom::Atom,
    rules::rule_core::RuleCore,
    rules::utils::VariableSource,
    rules::{GenericRule, JoinRule, ProductRule, ProjectRule},
    Annotation,
};

#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum Rule {
    Generic(GenericRule),
    Project(ProjectRule),
    Product(ProductRule),
    Join(JoinRule),
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
            Rule::Product(rule) => rule.core(),
            Rule::Join(rule) => rule.core(),
        }
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        match self {
            Rule::Generic(rule) => rule.core_mut(),
            Rule::Project(rule) => rule.core_mut(),
            Rule::Product(rule) => rule.core_mut(),
            Rule::Join(rule) => rule.core_mut(),
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
            Rule::Project(_) | Rule::Product(_) | Rule::Join(_) => None,
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
            Rule::Product(rule) => write!(f, "{}", rule),
            Rule::Join(rule) => write!(f, "{}", rule),
        }
    }
}
