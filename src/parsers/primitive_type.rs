//! Provides parsers for primitive types.

use crate::parsed_types::{Name, PrimitiveType};
use crate::parsers::{parse_name, ParseResult, Span};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;

/// Parses a primitive type, i.e. `object | <name>`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_primitive_type, preamble::*};
/// assert!(parse_primitive_type(Span::new("object")).is_value("object".into()));
/// assert!(parse_primitive_type(Span::new("number")).is_value("number".into()));
/// assert!(parse_primitive_type(Span::new("a-1_2")).is_value("a-1_2".into()));
/// assert!(parse_primitive_type(Span::new("obj!ect")).is_value("obj".into()));
///```
pub fn parse_primitive_type<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, PrimitiveType> {
    map(alt((parse_object, parse_name)), PrimitiveType::from)(input.into())
}

fn parse_object(input: Span) -> ParseResult<Name> {
    map(tag("object"), |x: Span| Name::from(*x.fragment()))(input)
}

impl crate::parsers::Parser for PrimitiveType {
    type Item = PrimitiveType;

    /// See [`parse_primitive_type`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_primitive_type(input)
    }
}
