//! Provides parsers for parsing a plan.

use crate::{
    parsed_types::Plan,
    parsers::{parse_plan_step, space_separated_list0, ParseResult, Span},
};
use nom::combinator::map;

/// Parses a plan.
///
/// ## Example
/// ```
/// # use lazylifted::parsed_types::*;
/// # use lazylifted::parsers::{parse_plan, preamble::*};
/// let input = r#"(pickup b1)
/// (stack b1 b2)
/// ; cost = 2 (unit cost)
/// "#;
/// let plan = parse_plan(Span::new(input));
/// assert!(plan.is_value(Plan::new(vec![
///    PlanStep::new(ActionName::from_str("pickup"), vec![Name::new("b1")]),
///    PlanStep::new(ActionName::from_str("stack"), vec![Name::new("b1"), Name::new("b2")]),
/// ])));
pub fn parse_plan<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Plan> {
    map(space_separated_list0(parse_plan_step), Plan::new)(input.into())
}

impl crate::parsers::Parser for Plan {
    type Item = Plan;

    /// Parses a plan.
    ///
    /// ## See also
    /// See [`parse_plan`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_plan(input)
    }
}
