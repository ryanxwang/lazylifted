//! Provides parsers for atomic formula skeletons.

use crate::parsed_types::PredicateDefinition;
use crate::parsers::{leading_whitespace, parens, typed_list, ParseResult, Span};
use crate::parsers::{parse_predicate_name, parse_variable};
use nom::combinator::map;
use nom::sequence::tuple;

/// Parses an predicate definition, i.e. `(<predicate> <typed list (variable)>)`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_predicate_definition, Span, UnwrapValue};
/// # use lazylifted::*;
/// assert!(parse_predicate_definition(Span::new("(at ?x - physob ?y - location)")).is_value(
///     PredicateDefinition::new(
///         PredicateName::from("at"),
///         TypedList::from_iter([
///             Variable::from("x").to_typed("physob"),
///             Variable::from("y").to_typed("location")
///         ]))
/// ));
/// ```
pub fn parse_predicate_definition<'a, T: Into<Span<'a>>>(
    input: T,
) -> ParseResult<'a, PredicateDefinition> {
    map(
        parens(tuple((
            parse_predicate_name,
            leading_whitespace(typed_list(parse_variable)),
        ))),
        |tuple| PredicateDefinition::from(tuple),
    )(input.into())
}

impl crate::parsers::Parser for PredicateDefinition {
    type Item = PredicateDefinition;

    /// Parses a predicate definition.
    ///
    /// ## See also
    /// See [`parse_predicate_definition`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_predicate_definition(input)
    }
}
