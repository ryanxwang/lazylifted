//! Provides parsers for predicate names.

use crate::parsed_types::PredicateName;
use crate::parsers::{parse_name, ParseResult, Span};
use nom::combinator::map;

/// Parses a predicate name, i.e. `<name>`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_predicate_name, preamble::*};
/// assert!(parse_predicate_name(Span::new("abcde")).is_value("abcde".into()));
/// assert!(parse_predicate_name(Span::new("a-1_2")).is_value("a-1_2".into()));
/// assert!(parse_predicate_name(Span::new("Z01")).is_value("Z01".into()));
/// assert!(parse_predicate_name(Span::new("x-_-_")).is_value("x-_-_".into()));
///
/// assert!(parse_predicate_name(Span::new("")).is_err());
/// assert!(parse_predicate_name(Span::new(".")).is_err());
/// assert!(parse_predicate_name(Span::new("-abc")).is_err());
/// assert!(parse_predicate_name(Span::new("0124")).is_err());
/// assert!(parse_predicate_name(Span::new("-1")).is_err());
///```
pub fn parse_predicate_name<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, PredicateName> {
    map(parse_name, PredicateName::from)(input.into())
}

impl crate::parsers::Parser for PredicateName {
    type Item = PredicateName;

    /// Parses a predicate name.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::{PredicateName, Parser};
    /// let (_, value) = PredicateName::parse("abcde").unwrap();
    /// assert_eq!(value, "abcde".into());
    ///```
    ///
    /// See [`parse_predicate_name`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_predicate_name(input)
    }
}
