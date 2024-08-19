//! Provides parsers for problem definitions.

use crate::parsed_types::{Objects, Problem, Requirements};
use crate::parsers::{
    leading_whitespace, literal, parse_name, parse_objects_declaration, parse_requirements,
    prefix_expr, space_separated_list0, surrounding_whitespace, ParseResult, Span,
};
use nom::branch::alt;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};

/// Parses a problem definitions.
///
/// ## Example
/// ```
/// # use lazylifted::parsers::{parse_problem, preamble::*};
/// # use lazylifted::parsed_types::*;
/// let input = r#"(define (problem get-paid)
///         (:domain briefcase-world)
///         (:init (place home) (place office)
///                (object p) (object d) (object b)
///                (at B home) (at P home) (at D home) (in P))
///         (:goal (and (at B office) (at D office) (at P home)))
///     )"#;
///
/// let (remainder, problem) = parse_problem(input).unwrap();
///
/// assert!(remainder.is_empty());
/// assert_eq!(problem.name(), &Name::new("get-paid"));
/// assert_eq!(problem.domain(), &Name::new("briefcase-world"));
/// assert!(problem.requirements().is_empty());
/// assert_eq!(problem.init().len(), 9);
/// assert_eq!(problem.goals().len(), 3);
/// ```
pub fn parse_problem<'a, T: Into<Span<'a>>>(input: T) -> ParseResult<'a, Problem> {
    map(
        surrounding_whitespace(prefix_expr(
            "define",
            tuple((
                prefix_expr("problem", parse_name),
                preceded(multispace1, prefix_expr(":domain", parse_name)),
                opt(preceded(multispace1, parse_requirements)),
                opt(preceded(multispace1, parse_objects_declaration)),
                preceded(
                    multispace1,
                    prefix_expr(":init", space_separated_list0(literal(parse_name))),
                ),
                preceded(
                    multispace1,
                    prefix_expr(
                        ":goal",
                        leading_whitespace(alt((
                            prefix_expr("and", space_separated_list0(literal(parse_name))),
                            map(literal(parse_name), |cond| vec![cond]),
                        ))),
                    ),
                ),
            )),
        )),
        |(name, domain, reqs, objects, init, goal)| {
            Problem::new(
                name,
                domain,
                reqs.unwrap_or(Requirements::new([])), // TODO-someday Do we need to imply STRIPS if empty?
                objects.unwrap_or(Objects::default()),
                init,
                goal,
            )
        },
    )(input.into())
}

impl crate::parsers::Parser for Problem {
    type Item = Problem;

    /// Parses a problem definitions.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsers::Parser;
    /// # use lazylifted::parsed_types::*;
    /// let input = r#"(define (problem get-paid)
    ///         (:domain briefcase-world)
    ///         (:init (place home) (place office)
    ///                (object p) (object d) (object b)
    ///                (at B home) (at P home) (at D home) (in P))
    ///         (:goal (and (at B office) (at D office) (at P home)))
    ///     )"#;
    ///
    /// let (_, problem) = Problem::parse(input).unwrap();
    ///
    /// assert_eq!(problem.name(), &Name::new("get-paid"));
    /// assert_eq!(problem.domain(), &Name::new("briefcase-world"));
    /// assert!(problem.requirements().is_empty());
    /// assert_eq!(problem.init().len(), 9);
    /// assert_eq!(problem.goals().len(), 3);
    /// ```
    ///
    /// ## See also
    /// See [`parse_problem`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_problem(input)
    }
}
