//! Provides parsers for goal object declarations.

use crate::parsed_types::Objects;
use crate::parsers::{parse_name, prefix_expr, typed_list, ParseResult, Span};
use nom::combinator::map;

/// Parser for object declarations.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_objects_declaration, preamble::*};
/// # use lazylifted::*;
/// let input = "(:objects train1 train2)";
/// assert!(parse_objects_declaration(input).is_value(
///     Objects::from_iter([
///         Name::new("train1").to_typed(Type::OBJECT),
///         Name::new("train2").to_typed(Type::OBJECT),
///     ])
/// ));
/// ```
pub fn parse_objects_declaration<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Objects> {
    map(
        prefix_expr(":objects", typed_list(parse_name)),
        Objects::new,
    )(input.into())
}

impl crate::parsers::Parser for Objects {
    type Item = Objects;

    /// See [`parse_objects_declaration`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_objects_declaration(input)
    }
}
