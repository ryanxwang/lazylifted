//! Provides parsers for atoms.

use crate::parsed_types::Atom;
use crate::parsers::{leading_whitespace, parens, space_separated_list0};
use crate::parsers::{parse_predicate_name, ParseResult, Span};
use nom::combinator::map;
use nom::sequence::tuple;

/// Parses an atom, i.e. `(<predicate> t*)`.
///
/// ## Example
/// ```
/// # use nom::character::complete::alpha1;
/// # use lazylifted::parsers::{atom, Span, parse_name, UnwrapValue};
/// # use lazylifted::*;
/// assert!(atom(parse_name)(Span::new("(move a b)")).is_value(
///     Atom::new(PredicateName::from("move"), vec!["a".into(), "b".into()])
/// ));
/// ```
pub fn atom<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, Atom<O>>
where
    F: Clone + FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    map(
        parens(tuple((
            parse_predicate_name,
            leading_whitespace(space_separated_list0(inner)),
        ))),
        |tuple| Atom::new(tuple.0, tuple.1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::parse_term;

    #[test]
    fn it_works() {
        let input = "(can-move ?from-waypoint ?to-waypoint)";
        let (_, _effect) = atom(parse_term)(Span::new(input)).unwrap();
    }
}
