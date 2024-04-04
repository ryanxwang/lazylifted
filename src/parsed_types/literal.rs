//! Contains literals via the [`Literal`] type.

use crate::parsed_types::Atom;

/// An [`Atom`] or its negated value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal<T> {
    Positive(Atom<T>),
    Negative(Atom<T>),
}

impl<'a, T> Literal<T> {
    pub const fn new(atom: Atom<T>) -> Self {
        Self::Positive(atom)
    }

    pub const fn new_not(atom: Atom<T>) -> Self {
        Self::Negative(atom)
    }

    pub const fn is_negated(&self) -> bool {
        matches!(self, Self::Negative(..))
    }
}

impl<'a, T> From<Atom<T>> for Literal<T> {
    fn from(value: Atom<T>) -> Self {
        Literal::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsed_types::{Name, PredicateName, Term};
    use crate::parsers::{atom, parse_term, Span};

    #[test]
    fn positive_atom() {
        let input = "(on x y)";
        let (_, effect) = atom(parse_term)(Span::new(input)).unwrap();

        let literal: Literal<Term> = effect.into();
        assert_eq!(
            literal,
            Literal::Positive(Atom::new(
                PredicateName::from_str("on"),
                vec![
                    Term::new_name(Name::from("x")),
                    Term::new_name(Name::from("y"))
                ]
            ))
        )
    }
}
