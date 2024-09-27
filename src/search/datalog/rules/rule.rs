use std::fmt::Display;

use strum_macros::EnumIs;

use crate::search::datalog::{
    atom::Atom,
    rules::rule_core::RuleCore,
    rules::utils::{VariablePositionInEffect, VariableSource},
    rules::{GenericRule, JoinRule, ProductRule, ProjectRule, RuleIndex},
    Annotation,
};

pub trait RuleTrait {
    fn core(&self) -> &RuleCore;
    fn core_mut(&mut self) -> &mut RuleCore;
    fn cleanup_grounding_data(&mut self);

    fn index(&self) -> RuleIndex {
        self.core().index()
    }

    fn set_index(&mut self, index: RuleIndex) {
        self.core_mut().set_index(index);
    }

    fn effect(&self) -> &Atom {
        self.core().effect()
    }

    fn conditions(&self) -> &[Atom] {
        self.core().conditions()
    }

    /// Update the conditions of the rule. Please make sure that the variable
    /// source is also updated.
    fn set_condition(&mut self, conditions: Vec<Atom>) {
        self.core_mut().set_condition(conditions);
    }

    fn weight(&self) -> f64 {
        self.core().weight()
    }

    fn annotation(&self) -> &Annotation {
        self.core().annotation()
    }

    fn variable_position_in_effect(&self) -> &VariablePositionInEffect {
        self.core().variable_position_in_effect()
    }

    fn variable_source(&self) -> &VariableSource {
        self.core().variable_source()
    }

    fn variable_source_mut(&mut self) -> &mut VariableSource {
        self.core_mut().variable_source_mut()
    }

    /// Update the condition at the given index, will update the variable source
    /// as well. Only supports dropping constant arguments of the condition.
    fn update_single_condition(&mut self, condition: Atom, index: usize) {
        self.core_mut().update_single_condition(condition, index);
    }

    /// Merge some conditions together into the provided new condition. Will
    /// update variable source to point into the variable source of the rule
    /// creating the new condition when appropriate.
    fn merge_conditions(
        &mut self,
        condition_indices_to_merge: &[usize],
        new_condition: Atom,
        new_condition_variable_source: &VariableSource,
    ) {
        self.core_mut().merge_conditions(
            condition_indices_to_merge,
            new_condition,
            new_condition_variable_source,
        );
    }
}

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

    pub fn new_product(rule: ProductRule) -> Self {
        Self::Product(rule)
    }

    pub fn new_join(rule: JoinRule) -> Self {
        Self::Join(rule)
    }

    pub fn schema_index(&self) -> Option<usize> {
        match self {
            Rule::Generic(rule) => Some(rule.schema_index()),
            Rule::Project(_) | Rule::Product(_) | Rule::Join(_) => None,
        }
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

impl RuleTrait for Rule {
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

    fn cleanup_grounding_data(&mut self) {
        match self {
            Rule::Generic(rule) => rule.cleanup_grounding_data(),
            Rule::Project(rule) => rule.cleanup_grounding_data(),
            Rule::Product(rule) => rule.cleanup_grounding_data(),
            Rule::Join(rule) => rule.cleanup_grounding_data(),
        }
    }
}
