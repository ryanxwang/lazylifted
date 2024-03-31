//! Provides parsers for constant definitions.

use crate::parsed_types::Constants;
use crate::parsers::{parse_name, prefix_expr, typed_list, ParseResult, Span};
use nom::combinator::map;

/// Parses constant definitions, i.e. `(:constants <typed list (name)>)`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_constants, preamble::*};
/// # use lazylifted::*;
/// let input = "(:constants B P D - physob)";
/// assert!(parse_constants(input).is_value(
///     Constants::new(TypedList::from_iter([
///         Name::from("B").to_typed("physob"),
///         Name::from("P").to_typed("physob"),
///         Name::from("D").to_typed("physob"),
///     ]))
/// ));
/// ```
pub fn parse_constants<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Constants> {
    map(prefix_expr(":constants", typed_list(parse_name)), |vec| {
        Constants::new(vec)
    })(input.into())
}

impl crate::parsers::Parser for Constants {
    type Item = Constants;

    /// See [`parse_constants`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_constants(input)
    }
}
