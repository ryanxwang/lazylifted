//! Provides parsers for goal definitions.

use crate::parsed_types::PropCondition;
use crate::parsers::{atom, parse_term, ParseResult, Span};
use crate::parsers::{prefix_expr, space_separated_list0};
use nom::branch::alt;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence::{preceded, tuple};

/// Parser for goal definitions.
///
/// ## Examples
/// ```
/// # use lazylifted::parsers::{parse_prop_condition, preamble::*};
/// # use lazylifted::parsed_types::*;
/// // Atom
/// assert!(parse_prop_condition("(on ?x b1)").is_value(
///    PropCondition::new_atom(
///       Atom::new(
///          PredicateName::from("on"),
///          vec![Term::Variable("x".into()), Term::Name("b1".into())]
///       )
///    )
/// ));
///
/// // Literal
/// assert!(parse_prop_condition("(not (on ?x b1))").is_value(
///    PropCondition::new_not(
///       PropCondition::new_atom(Atom::new(
///          PredicateName::from("on"),
///          vec![Term::Variable("x".into()), Term::Name("b1".into())]
///       ))
///    )
/// ));
///
/// // Conjunction (and)
/// assert!(parse_prop_condition("(and (not (on ?x b1)) (on ?x b2))").is_value(
///     PropCondition::new_and([
///         PropCondition::new_not(
///             PropCondition::new_atom(Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b1".into())]
///             ))
///         ),
///         PropCondition::new_atom(
///             Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b2".into())]
///             )
///         )
///     ])
/// ));
///
/// // Disjunction (or)
/// assert!(parse_prop_condition("(or (not (on ?x b1)) (on ?x b2))").is_value(
///     PropCondition::new_or([
///         PropCondition::new_not(
///             PropCondition::new_atom(Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b1".into())]
///             ))
///         ),
///         PropCondition::new_atom(
///             Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b2".into())]
///             )
///         )
///     ])
/// ));
///
/// // Implication
/// assert!(parse_prop_condition("(imply (not (on ?x b1)) (on ?x b2))").is_value(
///     PropCondition::new_imply(
///         PropCondition::new_not(
///             PropCondition::new_atom(Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b1".into())]
///             ))
///         ),
///         PropCondition::new_atom(
///             Atom::new(
///                 PredicateName::from("on"),
///                 vec![Term::Variable("x".into()), Term::Name("b2".into())]
///             )
///         )
///     )
/// ));
///
/// // Equality
/// assert!(parse_prop_condition("(not (= ?x b1))").is_value(
///     PropCondition::new_not(
///         PropCondition::new_equality(
///             Term::Variable("x".into()),
///             Term::Name("b1".into())
///        )
///     )
/// ));
/// ```
pub fn parse_prop_condition<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, PropCondition> {
    let atom = map(atom(parse_term), PropCondition::new_atom);

    let and = map(
        prefix_expr("and", space_separated_list0(parse_prop_condition)),
        PropCondition::new_and,
    );

    // :disjunctive-preconditions
    let or = map(
        prefix_expr("or", space_separated_list0(parse_prop_condition)),
        PropCondition::new_or,
    );

    // :negative-preconditions
    let not = map(
        prefix_expr("not", parse_prop_condition),
        PropCondition::new_not,
    );

    // :disjunctive-preconditions
    let imply = map(
        prefix_expr(
            "imply",
            tuple((
                parse_prop_condition,
                preceded(multispace1, parse_prop_condition),
            )),
        ),
        PropCondition::new_imply_tuple,
    );

    let equality = map(
        prefix_expr("=", tuple((parse_term, preceded(multispace1, parse_term)))),
        |(a, b)| PropCondition::new_equality(a, b),
    );

    alt((atom, and, or, not, imply, equality))(input.into())
}

impl crate::parsers::Parser for PropCondition {
    type Item = PropCondition;

    /// See [`parse_prop_condition`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_prop_condition(input)
    }
}
