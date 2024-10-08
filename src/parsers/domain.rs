//! Provides parsers for domain definitions.

use crate::parsed_types::Domain;
use crate::parsers::{
    parse_action_definition, parse_constants, parse_name, parse_predicate_definition,
    parse_requirements, parse_type_definitions, prefix_expr, space_separated_list1,
    surrounding_whitespace, ParseResult, Span,
};
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};

/// Parses a domain definition.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_domain, preamble::*};
/// # use lazylifted::parsed_types::*;
/// let input = r#"
/// (define
///     (domain sokoban)
///     (:requirements :typing)
///     (:types location direction box)
///
///     (:constants down up left right - direction)
///
///     (:predicates
///          (at-robot ?l - location)
///          (at ?o - box ?l - location)
///          (adjacent ?l1 - location ?l2 - location ?d - direction)
///          (clear ?l - location)
///     )
///
///     (:action move
///         :parameters (?from - location ?to - location ?dir - direction)
///         :precondition (and (clear ?to) (at-robot ?from) (adjacent ?from ?to ?dir))
///         :effect (and (at-robot ?to) (not (at-robot ?from)))
///     )
///             
///     (:action push
///         :parameters  (?rloc - location ?bloc - location ?floc - location ?dir - direction ?b - box)
///         :precondition (and (at-robot ?rloc) (at ?b ?bloc) (clear ?floc)
///                       (adjacent ?rloc ?bloc ?dir) (adjacent ?bloc ?floc ?dir))
///
///         :effect (and (at-robot ?bloc) (at ?b ?floc) (clear ?bloc)
///                 (not (at-robot ?rloc)) (not (at ?b ?bloc)) (not (clear ?floc)))
///     )
///)"#;
///
/// let (remainder, domain) = parse_domain(input).unwrap();
///
/// assert!(remainder.is_empty());
/// assert_eq!(domain.name(), &Name::new("sokoban"));
/// assert_eq!(domain.requirements().len(), 1);
/// assert_eq!(domain.types().len(), 4);
/// assert_eq!(domain.constants().len(), 4);
/// assert_eq!(domain.predicates().len(), 4);
/// assert_eq!(domain.actions().len(), 2);
/// ```
pub fn parse_domain<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Domain> {
    map(
        surrounding_whitespace(prefix_expr(
            "define",
            tuple((
                prefix_expr("domain", parse_name),
                opt(preceded(multispace1, parse_requirements)),
                opt(preceded(multispace1, parse_type_definitions)),
                opt(preceded(multispace1, parse_constants)),
                opt(preceded(
                    multispace1,
                    prefix_expr(
                        ":predicates",
                        space_separated_list1(parse_predicate_definition),
                    ),
                )),
                opt(preceded(
                    multispace1,
                    space_separated_list1(parse_action_definition),
                )),
            )),
        )),
        |(name, requirements, types, constants, predicates, actions)| {
            Domain::builder(name, actions.unwrap_or_default())
                .with_requirements(requirements.unwrap_or_default())
                .with_types(types.unwrap_or_default())
                .with_constants(constants.unwrap_or_default())
                .with_predicates(predicates.unwrap_or_default())
        },
    )(input.into())
}

impl crate::parsers::Parser for Domain {
    type Item = Domain;

    /// Parses a domain definition.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsers::Parser;
    /// # use lazylifted::parsed_types::*;
    /// let input = r#"
    /// (define
    ///     (domain sokoban)
    ///     (:requirements :typing)
    ///     (:types location direction box)
    ///
    ///     (:constants down up left right - direction)
    ///
    ///     (:predicates
    ///          (at-robot ?l - location)
    ///          (at ?o - box ?l - location)
    ///          (adjacent ?l1 - location ?l2 - location ?d - direction)
    ///          (clear ?l - location)
    ///     )
    ///
    ///     (:action move
    ///         :parameters (?from - location ?to - location ?dir - direction)
    ///         :precondition (and (clear ?to) (at-robot ?from) (adjacent ?from ?to ?dir))
    ///         :effect (and (at-robot ?to) (not (at-robot ?from)))
    ///     )
    ///             
    ///     (:action push
    ///         :parameters  (?rloc - location ?bloc - location ?floc - location ?dir - direction ?b - box)
    ///         :precondition (and (at-robot ?rloc) (at ?b ?bloc) (clear ?floc)
    ///                       (adjacent ?rloc ?bloc ?dir) (adjacent ?bloc ?floc ?dir))
    ///
    ///         :effect (and (at-robot ?bloc) (at ?b ?floc) (clear ?bloc)
    ///                 (not (at-robot ?rloc)) (not (at ?b ?bloc)) (not (clear ?floc)))
    ///     )
    ///)"#;
    ///
    /// let (_, domain) = Domain::parse(input).unwrap();
    ///
    /// assert_eq!(domain.name(), &Name::new("sokoban"));
    /// assert_eq!(domain.requirements().len(), 1);
    /// assert_eq!(domain.types().len(), 4);
    /// assert_eq!(domain.constants().len(), 4);
    /// assert_eq!(domain.predicates().len(), 4);
    /// assert_eq!(domain.actions().len(), 2);
    /// ```
    ///
    /// ## See also
    /// See [`parse_domain`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_domain(input)
    }
}
