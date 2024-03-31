//! Provides parsers for constant definitions.

use crate::parsed_types::Types;
use crate::parsers::{parse_name, prefix_expr, typed_list, ParseResult, Span};
use nom::combinator::map;

/// Parses type definitions, i.e. `(:types <typed list (name)>)`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_type_definitions, preamble::*};
/// # use lazylifted::*;
/// let input = "(:types location physob)";
/// assert!(parse_type_definitions(input).is_value(
///     Types::new(TypedList::from_iter([
///         Typed::new(Name::from("location"), Type::OBJECT),
///         Typed::new(Name::from("physob"), Type::OBJECT),
///     ]))
/// ));
/// ```
pub fn parse_type_definitions<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Types> {
    map(prefix_expr(":types", typed_list(parse_name)), |vec| {
        Types::new(vec)
    })(input.into())
}

impl crate::parsers::Parser for Types {
    type Item = Types;

    /// See [`parse_type_definitions`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_type_definitions(input)
    }
}
