use crate::parsed_types::{Atom as ParsedAtom, Name, Term};
use crate::search::{Atom, Negatable, SmallTuple};
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AtomSchema {
    predicate_index: usize,
    arguments: Vec<SchemaArgument>,
}

impl AtomSchema {
    #[cfg(test)]
    /// This is only used for testing. [`AtomSchema`]s should always be created
    /// from the parsed task.
    pub fn new(predicate_index: usize, arguments: Vec<SchemaArgument>) -> Self {
        Self {
            predicate_index,
            arguments,
        }
    }

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

    #[inline(always)]
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

    /// Returns a new AtomSchema with the given arguments partially grounded.
    /// The i-th element of the `object_indices` slice contains the index of
    /// object used to ground the schema parameter with index i.
    pub fn partially_ground(&self, object_indices: &[usize]) -> Self {
        Self {
            predicate_index: self.predicate_index,
            arguments: self
                .arguments
                .iter()
                .map(|arg| match arg {
                    SchemaArgument::Constant(index) => SchemaArgument::Constant(*index),
                    SchemaArgument::Free(index) => match object_indices.get(*index) {
                        Some(object_index) => SchemaArgument::Constant(*object_index),
                        None => SchemaArgument::Free(*index),
                    },
                })
                .collect(),
        }
    }

    pub fn ground(&self, object_indices: &[usize]) -> Atom {
        Atom::new(
            self.predicate_index,
            SmallTuple::new(
                self.arguments
                    .iter()
                    .map(|arg| match arg {
                        SchemaArgument::Constant(index) => *index,
                        SchemaArgument::Free(index) => *object_indices.get(*index).unwrap(),
                    })
                    .collect(),
            ),
        )
    }

    pub fn includes(&self, atom: &Atom) -> bool {
        self.predicate_index == atom.predicate_index()
            && self.arguments.len() == atom.arguments().len()
            && self
                .arguments
                .iter()
                .enumerate()
                .all(|(index, schema_arg)| match schema_arg {
                    SchemaArgument::Constant(object_index) => {
                        *object_index == atom.arguments()[index]
                    }
                    SchemaArgument::Free(_) => true,
                })
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

    pub fn partially_ground(&self, object_indices: &[usize]) -> Self {
        Negatable::new(
            self.is_negative(),
            self.underlying().partially_ground(object_indices),
        )
    }

    pub fn ground(&self, object_indices: &[usize]) -> Negatable<Atom> {
        Negatable::new(self.is_negative(), self.underlying().ground(object_indices))
    }

    pub fn includes(&self, atom: &Atom) -> bool {
        self.underlying().includes(atom)
    }
}
