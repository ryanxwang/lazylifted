//! Contains atoms via the [`Atom`] type.

use crate::parsed_types::PredicateName;
use std::ops::Deref;

/// An atom.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Atom<T> {
    predicate_name: PredicateName,
    values: Vec<T>,
}

impl<T> Atom<T> {
    pub const fn new(predicate_name: PredicateName, values: Vec<T>) -> Self {
        Self {
            predicate_name,
            values,
        }
    }

    /// Returns the predicate name.
    pub const fn predicate_name(&self) -> &PredicateName {
        &self.predicate_name
    }

    /// Gets a reference to the values.
    pub fn values(&self) -> &[T] {
        self.values.as_slice()
    }
}

impl<'a, T> From<(PredicateName, Vec<T>)> for Atom<T> {
    fn from(value: (PredicateName, Vec<T>)) -> Self {
        Atom::new(value.0, value.1)
    }
}

impl<'a, T> Deref for Atom<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.values()
    }
}
