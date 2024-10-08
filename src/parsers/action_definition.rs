//! Provides parsers for action definitions.

use crate::parsed_types::ActionDefinition;
use crate::parsers::{
    leading_whitespace, parens, parse_action_name, parse_prop_condition, parse_prop_effect,
    parse_variable, prefix_expr, space_separated_list0, typed_list, ParseResult, Span,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, tuple};

/// Parses an action definition.
///
/// ## Example
/// ```
/// # use lazylifted::parsed_types::*;
/// # use lazylifted::parsers::{parse_action_definition, preamble::*};
/// let input = r#"(:action putdown
///                    :parameters  (?ob)
///                    :precondition (holding ?ob)
///                    :effect (and (clear ?ob) (arm-empty) (on-table ?ob)
///                        (not (holding ?ob))))"#;
///
/// let action = parse_action_definition(Span::new(input));
///
/// assert!(action.is_value(
///     ActionDefinition::new(
///         ActionName::from_str("putdown"),
///         TypedList::from_iter([
///             Variable::from_str("ob").to_typed(Type::default()),
///         ]),
///         vec![
///             PropCondition::new_atom(Atom::new(
///                 PredicateName::from_str("holding"),
///                 vec![Term::Variable(Variable::from_str("ob"))])),
///         ],
///         vec![
///             PropEffect::new_add(Atom::new(
///                 PredicateName::from_str("clear"),
///                 vec![Term::Variable(Variable::from_str("ob"))])),
///             PropEffect::new_add(Atom::new(
///                 PredicateName::from_str("arm-empty"),
///                 vec![])),
///             PropEffect::new_add(Atom::new(
///                 PredicateName::from_str("on-table"),
///                 vec![Term::Variable(Variable::from_str("ob"))])),
///             PropEffect::new_delete(Atom::new(
///                 PredicateName::from_str("holding"),
///                 vec![Term::Variable(Variable::from_str("ob"))])),
///         ]
///     )
/// ));
/// ```
pub fn parse_action_definition<'a, T: Into<Span<'a>>>(
    input: T,
) -> ParseResult<'a, ActionDefinition> {
    let precondition = preceded(
        tag(":precondition"),
        preceded(
            multispace1,
            alt((
                prefix_expr("and", space_separated_list0(parse_prop_condition)),
                map(parse_prop_condition, |cond| vec![cond]),
            )),
        ),
    );
    let effect = preceded(
        tag(":effect"),
        preceded(
            multispace1,
            alt((
                prefix_expr("and", space_separated_list0(parse_prop_effect)),
                map(parse_prop_effect, |effect| vec![effect]),
            )),
        ),
    );
    let action_def_body = tuple((
        map(opt(leading_whitespace(precondition)), |pre| {
            pre.unwrap_or_default()
        }),
        map(opt(leading_whitespace(effect)), |eff| {
            eff.unwrap_or_default()
        }),
    ));
    let parameters = preceded(
        tag(":parameters"),
        preceded(multispace1, parens(typed_list(parse_variable))),
    );
    let action_def = prefix_expr(
        ":action",
        tuple((
            parse_action_name,
            preceded(multispace1, parameters),
            leading_whitespace(action_def_body),
        )),
    );

    map(action_def, |(symbol, params, (preconditions, effects))| {
        ActionDefinition::new(symbol, params, preconditions, effects)
    })(input.into())
}

impl crate::parsers::Parser for ActionDefinition {
    type Item = ActionDefinition;

    /// Parses an action definition.
    ///
    /// ## See also
    /// See [`parse_action_definition`].
    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item> {
        parse_action_definition(input)
    }
}
