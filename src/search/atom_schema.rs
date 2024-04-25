use crate::parsed_types::{Atom as ParsedAtom, Name, Term};
use crate::search::Negatable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AtomSchema {
    predicate_index: usize,
    arguments: Vec<SchemaArgument>,
}

impl AtomSchema {
    pub fn from_parsed(
        atom: &ParsedAtom<Term>,
        predicate_table: &HashMap<Name, usize>,
        parameter_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let predicate_index = *predicate_table
            .get(atom.predicate_name())
            .expect("Schema atom predicate not found in domain predicate table.");
        let arguments = atom
            .values()
            .iter()
            .map(|arg| SchemaArgument::new(arg, parameter_table, object_table))
            .collect();

        Self {
            predicate_index,
            arguments,
        }
    }

    pub fn is_nullary(&self) -> bool {
        self.arguments.is_empty()
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.predicate_index
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[SchemaArgument] {
        &self.arguments
    }

    #[inline(always)]
    pub fn argument(&self, index: usize) -> &SchemaArgument {
        &self.arguments[index]
    }
}

impl Negatable<AtomSchema> {
    pub fn new_atom_schema(
        atom: &ParsedAtom<Term>,
        negated: bool,
        predicate_table: &HashMap<Name, usize>,
        parameter_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        Negatable::new(
            negated,
            AtomSchema::from_parsed(atom, predicate_table, parameter_table, object_table),
        )
    }

    #[inline(always)]
    pub fn is_nullary(&self) -> bool {
        self.underlying().is_nullary()
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.underlying().predicate_index()
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[SchemaArgument] {
        self.underlying().arguments()
    }

    #[inline(always)]
    pub fn argument(&self, index: usize) -> &SchemaArgument {
        self.underlying().argument(index)
    }
}
