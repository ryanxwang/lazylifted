use crate::parsed_types::{
    ActionDefinition, ActionName, Name, PropCondition, PropEffect, Typed, Variable,
};
use crate::search::{Action, Atom, AtomSchema, Negatable, PartialAction};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaParameter {
    index: usize,
    type_index: usize,
}

impl SchemaParameter {
    pub fn new(index: usize, param: &Typed<Variable>, type_table: &HashMap<Name, usize>) -> Self {
        let param_type = param
            .type_()
            .get_primitive()
            .expect("Expecting primitive types in action parameters")
            .name();
        Self {
            index,
            type_index: *type_table.get(param_type).unwrap_or_else(|| {
                panic!(
                    "Schema parameter type {:?} not found in domain type table {:?}",
                    param_type, type_table
                )
            }),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn type_index(&self) -> usize {
        self.type_index
    }
}

#[derive(Debug, Clone)]
pub struct ActionSchema {
    name: ActionName,
    index: usize,
    parameters: Vec<SchemaParameter>,
    preconditions: Vec<Negatable<AtomSchema>>,
    effects: Vec<Negatable<AtomSchema>>,
}

impl ActionSchema {
    pub fn new(
        index: usize,
        action_definition: &ActionDefinition,
        predicate_table: &HashMap<Name, usize>,
        type_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let parameters: Vec<SchemaParameter> = action_definition
            .parameters()
            .iter()
            .enumerate()
            .map(|(index, param)| SchemaParameter::new(index, param, type_table))
            .collect();

        let parameter_table: HashMap<Name, usize> = action_definition
            .parameters()
            .iter()
            .enumerate()
            .map(|(index, param)| (param.value().name().clone(), index))
            .collect();

        let parameter_types: HashMap<usize, usize> = parameters
            .iter()
            .map(|param| (param.index(), param.type_index()))
            .collect();

        let mut preconditions = Vec::new();
        let mut effects = Vec::new();

        for precondition in action_definition.preconditions() {
            // let literal = match precondition {
            //     PropCondition::Literal(literal) => literal,
            //     _ => panic!("Expecting a literal prop condition"),
            // };
            let (atom, negated) = match precondition {
                PropCondition::Atom(atom) => (atom, false),
                PropCondition::Not(inner) => match inner.as_ref() {
                    PropCondition::Atom(atom) => (atom, true),
                    _ => panic!("Expecting a negated atom prop condition"),
                },
                _ => panic!("Expecting an atom prop condition"),
            };

            let atom_schema = Negatable::new_atom_schema(
                atom,
                negated,
                predicate_table,
                &parameter_table,
                &parameter_types,
                object_table,
            );
            preconditions.push(atom_schema);
        }

        for effect in action_definition.effects() {
            let (atom, negated) = match effect {
                PropEffect::Add(atom) => (atom, false),
                PropEffect::Delete(atom) => (atom, true),
            };

            let atom_schema = Negatable::new_atom_schema(
                atom,
                negated,
                predicate_table,
                &parameter_table,
                &parameter_types,
                object_table,
            );
            effects.push(atom_schema);
        }

        Self {
            name: action_definition.name().clone(),
            index,
            parameters,
            preconditions,
            effects,
        }
    }

    pub fn name(&self) -> &ActionName {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_ground(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn parameters(&self) -> &[SchemaParameter] {
        &self.parameters
    }

    pub fn preconditions(&self) -> &[Negatable<AtomSchema>] {
        &self.preconditions
    }

    pub fn effects(&self) -> &[Negatable<AtomSchema>] {
        &self.effects
    }

    pub fn partially_ground_effects(
        &self,
        partial_action: &PartialAction,
    ) -> Vec<Negatable<AtomSchema>> {
        assert!(partial_action.schema_index() == self.index);
        self.effects
            .iter()
            .map(|effect| effect.partially_ground(partial_action.partial_instantiation()))
            .collect()
    }

    pub fn ground_effects(&self, action: &Action) -> Vec<Negatable<Atom>> {
        assert!(action.index == self.index);
        self.effects
            .iter()
            .map(|effect| effect.ground(&action.instantiation))
            .collect()
    }

    /// Remove all the negative preconditions from the action schema, and update
    /// the effects to also add/delete the auxiliary negative predicates.
    pub fn update_with_auxiliary_negative_predicates(
        &mut self,
        original_predicate_to_negative_predicate: &HashMap<usize, usize>,
    ) {
        for precondition in &mut self.preconditions {
            if precondition.is_negative() {
                let predicate_index = precondition.predicate_index();
                let negative_predicate_index = *original_predicate_to_negative_predicate
                    .get(&predicate_index)
                    .expect("Negative predicate not found in auxiliary predicate table");
                *precondition =
                    precondition.negate_with_auxiliary_predicate(negative_predicate_index);
            }
        }

        let mut new_effects = Vec::new();
        for effect in &self.effects {
            if let Some(negative_predicate_index) =
                original_predicate_to_negative_predicate.get(&effect.predicate_index())
            {
                new_effects.push(effect.negate_with_auxiliary_predicate(*negative_predicate_index));
            }
        }

        self.effects.extend(new_effects);
    }
}

impl PartialEq for ActionSchema {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Display for ActionSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "((index {}) (parameters", self.index)?;
        for param in &self.parameters {
            write!(f, " ({} {})", param.index(), param.type_index())?;
        }
        write!(f, ") (preconditions")?;
        for precondition in &self.preconditions {
            write!(f, " {}", precondition)?;
        }
        write!(f, ") (effects")?;
        for effect in &self.effects {
            write!(f, " {}", effect)?;
        }
        write!(f, "))")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{small_tuple, Task},
        test_utils::*,
    };
    // use smallvec::smallvec;

    #[test]
    #[should_panic(expected = "assertion failed: action.index == self.index")]
    fn test_ground_effects_with_different_schema() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let _effects = task.action_schemas()[0].ground_effects(&Action::new(1, vec![3]));
    }

    #[test]
    fn test_ground_effects() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let effects = task.action_schemas()[0].ground_effects(&Action::new(0, vec![3]));

        assert_eq!(
            effects,
            vec![
                Negatable::Positive(Atom::new(3, small_tuple![3])),
                Negatable::Negative(Atom::new(0, small_tuple![3])),
                Negatable::Negative(Atom::new(1, small_tuple![3])),
                Negatable::Negative(Atom::new(2, small_tuple![])),
            ]
        )
    }
}
