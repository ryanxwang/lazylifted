use crate::parsed_types::{Atom as ParsedAtom, Name};
use crate::search::Negatable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An [`Atom`] is similar to a [`crate::search::AtomSchema`], except that
/// state atoms are fully grounded.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Atom {
    predicate_index: usize,
    arguments: Vec<usize>,
}

impl Atom {
    pub fn new(predicate_index: usize, arguments: Vec<usize>) -> Self {
        Self {
            predicate_index,
            arguments,
        }
    }

    pub fn from_parsed(
        atom: &ParsedAtom<Name>,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let predicate_index = *predicate_table
            .get(atom.predicate_name())
            .expect("Goal atom predicate not found in domain predicate table.");
        let arguments = atom
            .values()
            .iter()
            .map(|name| {
                *object_table
                    .get(name)
                    .expect("Goal atom argument not found in object table.")
            })
            .collect();

        Self {
            predicate_index,
            arguments,
        }
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.predicate_index
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[usize] {
        &self.arguments
    }
}

impl Negatable<Atom> {
    pub fn new_atom(
        atom: &ParsedAtom<Name>,
        negated: bool,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        Negatable::new(
            negated,
            Atom::from_parsed(atom, predicate_table, object_table),
        )
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.underlying().predicate_index()
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[usize] {
        self.underlying().arguments()
    }
}
