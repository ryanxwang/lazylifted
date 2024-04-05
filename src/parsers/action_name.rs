//! Provides parsers for action names.

use crate::parsed_types::ActionName;
use crate::parsers::{parse_name, ParseResult, Span};
use nom::combinator::map;

/// Parses an action symbol, i.e. `<name>`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_action_name, preamble::*};
/// assert!(parse_action_name(Span::new("abcde")).is_value("abcde".into()));
/// assert!(parse_action_name(Span::new("a-1_2")).is_value("a-1_2".into()));
/// assert!(parse_action_name(Span::new("Z01")).is_value("Z01".into()));
/// assert!(parse_action_name(Span::new("x-_-_")).is_value("x-_-_".into()));
///
/// assert!(parse_action_name(Span::new("")).is_err());
/// assert!(parse_action_name(Span::new(".")).is_err());
/// assert!(parse_action_name(Span::new("-abc")).is_err());
/// assert!(parse_action_name(Span::new("0124")).is_err());
/// assert!(parse_action_name(Span::new("-1")).is_err());
///```
pub fn parse_action_name<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, ActionName> {
    map(parse_name, ActionName::new)(input.into())
}

impl crate::parsers::Parser for ActionName {
    type Item = ActionName;

    /// Parses an action symbol.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsers::Parser;
    /// # use lazylifted::parsed_types::ActionName;
    /// let (_, action_name) = ActionName::parse("abcde").unwrap();
    /// assert_eq!(action_name, "abcde".into());
    ///```
    ///
    /// ## See also
    /// See [`parse_action_name`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_action_name(input)
    }
}
