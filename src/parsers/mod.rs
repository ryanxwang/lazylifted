mod action_definition;
mod action_name;
mod atom;
mod comments;
mod constants;
mod domain;
mod literal;
mod name;
mod object_declarations;
mod plan;
mod plan_step;
mod predicate_definition;
mod predicate_name;
mod primitive_type;
mod problem;
mod prop_condition;
mod prop_effect;
mod requirements;
mod term;
mod test_helpers;
mod r#type;
mod type_definitions;
mod typed_list;
mod utilities;
mod variable;

#[cfg(test)]
pub(crate) use test_helpers::Match;
pub use test_helpers::UnwrapValue;

pub trait Parser {
    type Item;

    fn parse<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, Self::Item>;

    fn parse_span(input: Span) -> ParseResult<Self::Item> {
        Self::parse(input)
    }

    /// Parse a string slice into the desired type. Discards any remaining
    /// input.
    fn from_str(input: &str) -> Result<Self::Item, nom::Err<ParseError>> {
        let (_, value) = Self::parse(input)?;
        Ok(value)
    }
}

pub type Span<'a> = nom_locate::LocatedSpan<&'a str>;

pub type ParseError<'a> = nom_greedyerror::GreedyError<Span<'a>, nom::error::ErrorKind>;

pub type ParseResult<'a, T, E = ParseError<'a>> = nom::IResult<Span<'a>, T, E>;

/// Re-exports commonly used types.
pub mod preamble {
    pub use crate::parsers::test_helpers::UnwrapValue;
    pub use crate::parsers::Parser;
    pub use crate::parsers::{ParseError, ParseResult, Span};
}

// Parsers
pub use action_definition::parse_action_definition;
pub use action_name::parse_action_name;
pub use comments::ignore_single_line_comment;
pub use constants::parse_constants;
pub use domain::parse_domain;
pub use name::parse_name;
pub use object_declarations::parse_objects_declaration;
pub use plan::parse_plan;
pub use plan_step::parse_plan_step;
pub use predicate_definition::parse_predicate_definition;
pub use predicate_name::parse_predicate_name;
pub use primitive_type::parse_primitive_type;
pub use problem::parse_problem;
pub use prop_condition::parse_prop_condition;
pub use prop_effect::parse_prop_effect;
pub use r#type::parse_type;
pub use requirements::{parse_requirement_key, parse_requirements};
pub use term::parse_term;
pub use type_definitions::parse_type_definitions;
pub use variable::parse_variable;

// Parser combinators
pub use atom::atom;
pub use literal::literal;
pub use typed_list::typed_list;

#[allow(unused_imports)]
pub(crate) use utilities::{
    leading_whitespace, parens, prefix_expr, space_separated_list0, space_separated_list1,
    surrounding_whitespace,
};
