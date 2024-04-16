//! Provides parsers for terms.

use crate::parsed_types::Term;
use crate::parsers::{parse_name, parse_variable, ParseResult, Span};
use nom::error::ErrorKind;
use nom::error_position;

/// Parses a term, i.e. `<name> | <variable>`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_term, preamble::*};
/// # use lazylifted::parsed_types::Term;
/// assert!(parse_term("abcde").is_value(Term::Name("abcde".into())));
/// assert!(parse_term("?abcde").is_value(Term::Variable("abcde".into())));
///```
pub fn parse_term<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Term> {
    let input = input.into();

    if let Ok((remaining, variable)) = parse_variable(input) {
        return Ok((remaining, Term::Variable(variable)));
    }

    if let Ok((remaining, name)) = parse_name(input) {
        return Ok((remaining, Term::Name(name)));
    }

    Err(nom::Err::Error(error_position!(
        input,
        ErrorKind::Alt
    )))
}

impl crate::parsers::Parser for Term {
    type Item = Term;

    /// Parses a term.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsers::Parser;
    /// # use lazylifted::parsed_types::Term;
    /// let (_, value) = Term::parse("some-name").unwrap();
    /// assert_eq!(value, Term::Name("some-name".into()));
    ///
    /// let (_, value) = Term::parse("?some-var").unwrap();
    /// assert_eq!(value, Term::Variable("some-var".into()));
    ///```
    ///
    /// ## See also
    /// See [`parse_term`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_term(input)
    }
}
