use crate::parsed_types::{
    ActionDefinition, ActionName, Literal, Name, PropCondition, PropEffect, Typed, Variable,
};
use crate::search::{Action, Atom, AtomSchema, Negatable, PartialAction};
use std::collections::HashMap;

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
        let parameters = action_definition
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

        let mut preconditions = Vec::new();
        let mut effects = Vec::new();

        for precondition in action_definition.preconditions() {
            let literal = match precondition {
                PropCondition::Literal(literal) => literal,
                _ => panic!("Expecting a literal prop condition"),
            };
            let (atom, negated) = match literal {
                Literal::Positive(atom) => (atom, false),
                Literal::Negative(atom) => (atom, true),
            };

            let atom_schema = Negatable::new_atom_schema(
                atom,
                negated,
                predicate_table,
                &parameter_table,
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
}

impl PartialEq for ActionSchema {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
