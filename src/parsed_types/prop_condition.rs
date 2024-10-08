//! Contains propositional condition definitions via the [`PropCondition`] type.

use crate::parsed_types::{Atom, Term};

/// A condition definition.
#[derive(Debug, Clone, PartialEq)]
pub enum PropCondition {
    Atom(Atom<Term>),
    And(Vec<PropCondition>),
    /// ## Requirements
    /// Requires [Disjunctive Preconditions](crate::parsed_types::Requirement::DisjunctivePreconditions).
    Or(Vec<PropCondition>),
    /// ## Requirements
    /// Requires [Negative Preconditions](crate::parsed_types::Requirement::NegativePreconditions).
    Not(Box<PropCondition>),
    /// ## Requirements
    /// Requires [Disjunctive Preconditions](crate::parsed_types::Requirement::DisjunctivePreconditions).
    Imply(Box<PropCondition>, Box<PropCondition>),
    /// ## Requirements
    /// Requires [Equality](crate::parsed_types::Requirement::Equality).
    Equality(Term, Term),
}

impl PropCondition {
    #[inline(always)]
    pub const fn new_atom(value: Atom<Term>) -> Self {
        Self::Atom(value)
    }

    #[inline(always)]
    pub fn new_and<T: IntoIterator<Item = PropCondition>>(values: T) -> Self {
        // TODO-someday Flatten `(and (and a b) (and x y))` into `(and a b c
        // y)`.
        Self::And(values.into_iter().collect())
    }

    #[inline(always)]
    pub fn new_or<T: IntoIterator<Item = PropCondition>>(values: T) -> Self {
        // TODO-someday Flatten `(or (or a b) (or x y))` into `(or a b c y)`.
        Self::Or(values.into_iter().collect())
    }

    #[inline(always)]
    pub fn new_not(value: PropCondition) -> Self {
        Self::Not(Box::new(value))
    }

    #[inline(always)]
    pub fn new_imply_tuple(tuple: (PropCondition, PropCondition)) -> Self {
        Self::new_imply(tuple.0, tuple.1)
    }

    #[inline(always)]
    pub fn new_imply(a: PropCondition, b: PropCondition) -> Self {
        Self::Imply(Box::new(a), Box::new(b))
    }

    #[inline(always)]
    pub const fn new_equality(a: Term, b: Term) -> Self {
        Self::Equality(a, b)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            PropCondition::Atom(_) => false,
            PropCondition::And(x) => x.iter().all(|y| y.is_empty()),
            PropCondition::Or(x) => x.iter().all(|y| y.is_empty()),
            PropCondition::Not(x) => x.is_empty(),
            PropCondition::Imply(x, y) => x.is_empty() && y.is_empty(),
            PropCondition::Equality(_, _) => false,
        }
    }
}
