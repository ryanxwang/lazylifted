//! Provides parsers for parsing a plan.

use crate::{
    parsed_types::PlanStep,
    parsers::{parens, parse_action_name, parse_name, space_separated_list0, ParseResult, Span},
};
use nom::{combinator::map, sequence::tuple};

/// Parses a single step of a plan.
///
/// ## Example
/// ```
/// # use lazylifted::parsed_types::*;
/// # use lazylifted::parsers::{parse_plan_step, preamble::*};
/// let input = "(stack b1 b2)";
/// let plan_step = parse_plan_step(Span::new(input));
/// assert!(plan_step.is_value(PlanStep::new(
///    ActionName::from_str("stack"),
///    vec![
///        Name::new("b1"),
///        Name::new("b2"),
///    ]
/// )));
pub fn parse_plan_step<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, PlanStep> {
    map(
        parens(tuple((
            parse_action_name,
            space_separated_list0(parse_name),
        ))),
        |(action_name, parameters)| PlanStep::new(action_name, parameters),
    )(input.into())
}

impl crate::parsers::Parser for PlanStep {
    type Item = PlanStep;

    /// Parses a plan ste.
    ///
    /// ## See also
    /// See [`parse_plan_step`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_plan_step(input)
    }
}
