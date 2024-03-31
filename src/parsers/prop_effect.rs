//! Provides parsers for propositional effects.

use crate::parsed_types::PropEffect;
use crate::parsers::{atom, parse_term, prefix_expr, ParseResult, Span};
use nom::branch::alt;
use nom::combinator::map;

/// Parses propositional effects.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_prop_effect, preamble::*};
/// # use lazylifted::*;
/// assert!(parse_prop_effect("(on ?x b1)").is_value(
///     PropEffect::new_add(Atom::new(
///        PredicateName::from("on"),
///        vec![Term::Variable("x".into()), Term::Name("b1".into())]
///     ))
/// ));
///
/// assert!(parse_prop_effect("(not (on ?x b1))").is_value(
///     PropEffect::new_delete(Atom::new(
///        PredicateName::from("on"),
///        vec![Term::Variable("x".into()), Term::Name("b1".into())]
///     ))
/// ));
/// ```
pub fn parse_prop_effect<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, PropEffect> {
    let is = map(atom(parse_term), |af| PropEffect::new_add(af));
    let is_not = map(prefix_expr("not", atom(parse_term)), |af| {
        PropEffect::new_delete(af)
    });

    alt((is_not, is))(input.into())
}

impl crate::parsers::Parser for PropEffect {
    type Item = PropEffect;

    /// See [`parse_prop_effect`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_prop_effect(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let input = "(can-move ?from-waypoint ?to-waypoint)";
        let (_, _effect) = parse_prop_effect(Span::new(input)).unwrap();
    }

    #[test]
    fn not_works() {
        let input = "(not (at B ?m))";
        let mut is_not = map(prefix_expr("not", atom(parse_term)), |af| {
            PropEffect::new_delete(af)
        });

        let result = is_not(input.into());
        assert!(result.is_ok());
    }
}
