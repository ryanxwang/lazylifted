//! Contains definition for a single proposition effect.

use crate::parsed_types::{Atom, Term};

/// A propositional effect.
#[derive(Debug, Clone, PartialEq)]
pub enum PropEffect {
    Add(Atom<Term>),
    Delete(Atom<Term>),
}

impl PropEffect {
    pub const fn new_add(atom: Atom<Term>) -> Self {
        Self::Add(atom)
    }

    pub const fn new_delete(atom: Atom<Term>) -> Self {
        Self::Delete(atom)
    }
}
