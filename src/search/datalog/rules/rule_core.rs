use std::{collections::HashMap, fmt::Display};

use crate::search::datalog::{
    atom::Atom,
    rules::utils::{VariablePositionInBody, VariablePositionInEffect, VariableSource},
    Annotation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleIndex(usize);

impl RuleIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

/// A [`RuleCore`] represents the core of a Datalog rule. This should be used as
/// a component of a [`Rule`](crate::search::datalog::rules::Rule).
#[derive(Debug, Clone)]
pub struct RuleCore {
    /// The index of the rule
    index: Option<RuleIndex>,
    /// The effect of the rule.
    effect: Atom,
    /// The conditions of the rule.
    conditions: Vec<Atom>,
    /// The weight of the rule.
    weight: f64,
    /// The annotation of the rule.
    annotation: Annotation,
    /// The mapping of variables to their positions in the effect atom.
    variable_position_in_effect: VariablePositionInEffect,
    /// The lookup table for variables in the rule.
    variable_source: VariableSource,
}

impl RuleCore {
    /// Create a new [`RuleCore`] with the given effect, conditions, weight, and
    /// annotation.
    pub fn new(effect: Atom, conditions: Vec<Atom>, weight: f64, annotation: Annotation) -> Self {
        assert!(
            !conditions.is_empty(),
            "Datalog rules cannot have empty condition"
        );
        let variable_position_in_effect = VariablePositionInEffect::new(&effect);
        let variable_source = VariableSource::new(&effect, &conditions);
        Self {
            // Indices get actually assigned after all the program preprocessing
            // is done
            index: None,
            effect,
            conditions,
            weight,
            annotation,
            variable_position_in_effect,
            variable_source,
        }
    }

    #[inline(always)]
    pub fn index(&self) -> RuleIndex {
        self.index.unwrap()
    }

    #[inline(always)]
    pub fn set_index(&mut self, index: RuleIndex) {
        self.index = Some(index);
    }

    /// Get the effect of the rule.
    #[inline(always)]
    pub fn effect(&self) -> &Atom {
        &self.effect
    }

    /// Get the conditions of the rule.
    #[inline(always)]
    pub fn conditions(&self) -> &[Atom] {
        &self.conditions
    }

    #[inline(always)]
    pub fn set_condition(&mut self, conditions: Vec<Atom>) {
        self.conditions = conditions;
    }

    #[inline(always)]
    pub fn weight(&self) -> f64 {
        self.weight
    }

    #[inline(always)]
    pub fn annotation(&self) -> &Annotation {
        &self.annotation
    }

    #[inline(always)]
    pub fn variable_source(&self) -> &VariableSource {
        &self.variable_source
    }

    #[inline(always)]
    pub fn variable_source_mut(&mut self) -> &mut VariableSource {
        &mut self.variable_source
    }

    #[inline(always)]
    pub fn variable_position_in_effect(&self) -> &VariablePositionInEffect {
        &self.variable_position_in_effect
    }

    pub fn update_single_condition(&mut self, condition: Atom, index: usize) {
        let new_argument_index: HashMap<usize, usize> = condition
            .arguments()
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                assert!(arg.is_variable());
                (arg.index(), i)
            })
            .collect();

        for table_index in 0..self.variable_source.table().len() {
            if self.variable_source.table()[table_index].condition_index() == index {
                let variable_index = self
                    .variable_source
                    .get_variable_index_from_table_index(table_index);
                match self.variable_source.table()[table_index] {
                    VariablePositionInBody::Direct {
                        condition_index, ..
                    } => {
                        self.variable_source.table_mut()[table_index] =
                            VariablePositionInBody::Direct {
                                condition_index,
                                argument_index: new_argument_index[&variable_index],
                            };
                    }
                    VariablePositionInBody::Indirect {
                        condition_index, ..
                    } => {
                        panic!(
                            "Cannot update indirect variable position in body: {}",
                            condition_index
                        );
                    }
                }
            }
        }

        self.conditions[index] = condition;
    }

    pub fn merge_conditions(
        &mut self,
        condition_indices_to_merge: &[usize],
        new_condition: Atom,
        new_condition_variable_source: &VariableSource,
    ) {
        // we shift all the original conditions forward, and then insert the new
        // condition at the end
        let mut new_conditions =
            Vec::with_capacity(self.conditions.len() - condition_indices_to_merge.len() + 1);
        let mut old_condition_index_to_new = HashMap::new();
        for (i, condition) in self.conditions.iter().enumerate() {
            if condition_indices_to_merge.contains(&i) {
                continue;
            }
            new_conditions.push(condition.clone());
            old_condition_index_to_new.insert(i, new_conditions.len() - 1);
        }
        new_conditions.push(new_condition);
        self.conditions = new_conditions;

        // update the variable source
        for table_index in 0..self.variable_source.table().len() {
            let condition_index = self.variable_source.table()[table_index].condition_index();
            if condition_indices_to_merge.contains(&condition_index) {
                let variable_index = self
                    .variable_source
                    .get_variable_index_from_table_index(table_index);
                let table_index_in_indirect_source = new_condition_variable_source
                    .get_table_index_from_variable_index(variable_index);
                self.variable_source.table[table_index] = VariablePositionInBody::Indirect {
                    condition_index: self.conditions.len() - 1,
                    table_index: table_index_in_indirect_source,
                };
            } else {
                let new_condition_index = old_condition_index_to_new[&condition_index];
                self.variable_source.table[table_index].set_condition_index(new_condition_index);
            }
        }
    }
}

impl PartialEq for RuleCore {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect
            && self.conditions == other.conditions
            && self.weight == other.weight
            && self.annotation == other.annotation
    }
}

impl Display for RuleCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <- ", self.effect)?;
        for (i, condition) in self.conditions.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", condition)?;
        }
        write!(
            f,
            "  | weight: {}; annotation: {}",
            self.weight, self.annotation
        )
    }
}
