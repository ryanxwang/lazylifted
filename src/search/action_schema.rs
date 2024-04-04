use crate::parsed_types::{
    ActionDefinition, ActionName, Atom, Literal, Name, PropCondition, PropEffect, Term, Typed,
    Variable,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
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
            type_index: type_table
                .get(param_type)
                .expect(
                    format!(
                        "Schema parameter type {:?} not found in domain type table {:?}",
                        param_type, type_table
                    )
                    .as_str(),
                )
                .clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// If the argument is a constant, then the value is the index of the object in
/// the task, otherwise the index is the index of the parameter in the action
/// schema.
pub enum SchemaArgument {
    Constant(usize),
    Free(usize),
}

impl SchemaArgument {
    pub fn new(
        argument: &Term,
        parameter_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        match argument {
            Term::Name(name) => {
                let index = object_table
                    .get(name)
                    .expect("Schema constant argument not found in object table.");
                Self::Constant(*index)
            }
            Term::Variable(var) => {
                let index = parameter_table
                    .get(var.name())
                    .expect("Schema variable argument not found in parameter table.");
                Self::Free(*index)
            }
        }
    }

    pub fn get_index(&self) -> usize {
        match self {
            Self::Constant(index) => *index,
            Self::Free(index) => *index,
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            Self::Constant(_) => true,
            Self::Free(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchemaAtom {
    pub predicate_index: usize,
    pub negated: bool,
    pub arguments: Vec<SchemaArgument>,
}

impl SchemaAtom {
    pub fn new(
        atom: &Atom<Term>,
        negated: bool,
        predicate_table: &HashMap<Name, usize>,
        parameter_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        debug_assert!(!atom.values().is_empty());

        let predicate_index = predicate_table
            .get(atom.predicate_name())
            .expect("Schema atom predicate not found in domain predicate table.")
            .clone();
        let arguments = atom
            .values()
            .iter()
            .map(|arg| SchemaArgument::new(arg, parameter_table, object_table))
            .collect();

        Self {
            predicate_index,
            negated,
            arguments,
        }
    }

    pub fn is_nullary(&self) -> bool {
        self.arguments.is_empty()
    }
}

#[derive(Debug)]
pub struct ActionSchema {
    pub name: ActionName,
    pub index: usize,
    pub parameters: Vec<SchemaParameter>,
    pub preconditions: Vec<SchemaAtom>,
    pub positive_nullary_preconditions: Vec<bool>,
    pub negative_nullary_preconditions: Vec<bool>,
    pub effects: Vec<SchemaAtom>,
    pub positive_nullary_effects: Vec<bool>,
    pub negative_nullary_effects: Vec<bool>,
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
        let mut positive_nullary_preconditions = vec![false; predicate_table.len()];
        let mut negative_nullary_preconditions = vec![false; predicate_table.len()];
        let mut effects = Vec::new();
        let mut positive_nullary_effects = vec![false; predicate_table.len()];
        let mut negative_nullary_effects = vec![false; predicate_table.len()];

        for precondition in action_definition.preconditions() {
            let literal = match precondition {
                PropCondition::Literal(literal) => literal,
                _ => panic!("Expecting a literal prop condition"),
            };
            let (atom, negated) = match literal {
                Literal::Positive(atom) => (atom, false),
                Literal::Negative(atom) => (atom, true),
            };

            if atom.values().is_empty() {
                let pred_index = predicate_table
                    .get(atom.predicate_name())
                    .expect("Schema precondition predicate not found in domain predicate table.");
                if negated {
                    negative_nullary_preconditions[*pred_index] = true;
                } else {
                    positive_nullary_preconditions[*pred_index] = true;
                }
                continue;
            }

            let schema_atom = SchemaAtom::new(
                atom,
                negated,
                predicate_table,
                &parameter_table,
                object_table,
            );
            preconditions.push(schema_atom);
        }

        for effect in action_definition.effects() {
            let (atom, negated) = match effect {
                PropEffect::Add(atom) => (atom, false),
                PropEffect::Delete(atom) => (atom, true),
            };

            if atom.values().is_empty() {
                let pred_index = predicate_table
                    .get(atom.predicate_name())
                    .expect("Schema precondition predicate not found in domain predicate table.");
                if negated {
                    negative_nullary_effects[*pred_index] = true;
                } else {
                    positive_nullary_effects[*pred_index] = true;
                }
                continue;
            }

            let schema_atom = SchemaAtom::new(
                atom,
                negated,
                predicate_table,
                &parameter_table,
                object_table,
            );
            effects.push(schema_atom);
        }

        Self {
            name: action_definition.name().clone(),
            index,
            parameters,
            preconditions,
            positive_nullary_preconditions,
            negative_nullary_preconditions,
            effects,
            positive_nullary_effects,
            negative_nullary_effects,
        }
    }

    pub fn is_ground(&self) -> bool {
        self.parameters.is_empty()
    }
}
