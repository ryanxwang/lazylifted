//! Provides parsers for literals.

use crate::parsed_types::Literal;
use crate::parsers::prefix_expr;
use crate::parsers::{atom, ParseResult, Span};
use nom::branch::alt;
use nom::combinator::map;

/// Parser combinator that parses a literal, i.e. `<atomic formula(t)> | (not <atomic formula(t)>)`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{literal, parse_name, preamble::*};
/// # use lazylifted::*;
/// assert!(literal(parse_name)(Span::new("(on b1 b2)")).is_value(
///     Literal::new(
///         Atom::new(
///             PredicateName::from("on"),
///             vec![Name::from("b1"), Name::from("b2")]
///         )
///     )
/// ));
/// assert!(literal(parse_name)(Span::new("(not (on b1 b2))")).is_value(
///     Literal::new_not(
///         Atom::new(
///             PredicateName::from("on"),
///             vec![Name::from("b1"), Name::from("b2")]
///         )
///     )
/// ));
/// ```
pub fn literal<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, Literal<O>>
where
    F: Clone + FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    let is = map(atom(inner.clone()), |af| Literal::new(af));
    let is_not = map(prefix_expr("not", atom(inner)), |af| Literal::new_not(af));

    alt((is_not, is))
}
