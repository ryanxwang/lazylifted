//! Provides parsers for requirements.

use crate::parsed_types::requirement::{names, Requirement};
use crate::parsed_types::Requirements;
use crate::parsers::{prefix_expr, space_separated_list1, ParseResult, Span};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;

/// Parses a requirement definition, i.e. `(:requirements <require-key>)⁺`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_requirements, preamble::*};
/// # use lazylifted::parsed_types::{Requirement, Requirements};
/// assert!(parse_requirements("(:requirements :adl)").is_value(Requirements::new([Requirement::Adl])));
/// assert!(parse_requirements("(:requirements :strips :typing)").is_value(Requirements::new([Requirement::Strips, Requirement::Typing])));
/// assert!(parse_requirements("(:requirements\n:strips   :typing  )").is_value(Requirements::new([Requirement::Strips, Requirement::Typing])));
///```
pub fn parse_requirements<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Requirements> {
    map(
        prefix_expr(
            ":requirements",
            space_separated_list1(parse_requirement_key),
        ),
        Requirements::new,
    )(input.into())
}

/// Parses a requirement key, i.e. `:strips`.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_requirement_key, preamble::*};
/// # use lazylifted::parsed_types::Requirement;
/// assert!(parse_requirement_key(":strips").is_value(Requirement::Strips));
/// assert!(parse_requirement_key(":typing").is_value(Requirement::Typing));
/// assert!(parse_requirement_key(":negative-preconditions").is_value(Requirement::NegativePreconditions));
/// assert!(parse_requirement_key(":disjunctive-preconditions").is_value(Requirement::DisjunctivePreconditions));
/// assert!(parse_requirement_key(":equality").is_value(Requirement::Equality));
/// assert!(parse_requirement_key(":existential-preconditions").is_value(Requirement::ExistentialPreconditions));
/// assert!(parse_requirement_key(":universal-preconditions").is_value(Requirement::UniversalPreconditions));
/// assert!(parse_requirement_key(":quantified-preconditions").is_value(Requirement::QuantifiedPreconditions));
/// assert!(parse_requirement_key(":conditional-effects").is_value(Requirement::ConditionalEffects));
/// assert!(parse_requirement_key(":fluents").is_value(Requirement::Fluents));
/// assert!(parse_requirement_key(":numeric-fluents").is_value(Requirement::NumericFluents));
/// assert!(parse_requirement_key(":adl").is_value(Requirement::Adl));
/// assert!(parse_requirement_key(":durative-actions").is_value(Requirement::DurativeActions));
/// assert!(parse_requirement_key(":duration-inequalities").is_value(Requirement::DurationInequalities));
/// assert!(parse_requirement_key(":continuous-effects").is_value(Requirement::ContinuousEffects));
/// assert!(parse_requirement_key(":derived-predicates").is_value(Requirement::DerivedPredicates));
/// assert!(parse_requirement_key(":timed-initial-literals").is_value(Requirement::TimedInitialLiterals));
/// assert!(parse_requirement_key(":preferences").is_value(Requirement::Preferences));
/// assert!(parse_requirement_key(":constraints").is_value(Requirement::Constraints));
/// assert!(parse_requirement_key(":action-costs").is_value(Requirement::ActionCosts));
///
/// assert!(parse_requirement_key(":unknown").is_err());
/// assert!(parse_requirement_key("invalid").is_err());
///```
pub fn parse_requirement_key<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Requirement> {
    map(
        alt((
            tag(names::STRIPS),
            tag(names::TYPING),
            tag(names::NEGATIVE_PRECONDITIONS),
            tag(names::DISJUNCTIVE_PRECONDITIONS),
            tag(names::EQUALITY),
            tag(names::EXISTENTIAL_PRECONDITIONS),
            tag(names::UNIVERSAL_PRECONDITIONS),
            tag(names::QUANTIFIED_PRECONDITIONS),
            tag(names::CONDITIONAL_EFFECTS),
            tag(names::FLUENTS),
            tag(names::NUMERIC_FLUENTS),
            tag(names::OBJECT_FLUENTS),
            tag(names::ADL),
            tag(names::DURATIVE_ACTIONS),
            tag(names::DURATION_INEQUALITIES),
            tag(names::CONTINUOUS_EFFECTS),
            tag(names::DERIVED_PREDICATES),
            tag(names::TIMED_INITIAL_LITERALS),
            tag(names::PREFERENCES),
            tag(names::CONSTRAINTS),
            tag(names::ACTION_COSTS),
        )),
        |x: Span| Requirement::try_from(*x.fragment()).expect("unhandled variant"),
    )(input.into())
}

impl crate::parsers::Parser for Requirements {
    type Item = Requirements;

    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_requirements(input)
    }
}

impl crate::parsers::Parser for Requirement {
    type Item = Requirement;

    /// See [`parse_requirement_key`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_requirement_key(input)
    }
}
