//! Utility parsers.

use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded},
};

use crate::parsers::{ignore_single_line_comment, ParseResult, Span};

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes a leading `(name` and trailing `)`, returning the output of `inner`.
pub fn prefix_expr<'a, F, O>(name: &'a str, inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, O>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    delimited(
        preceded(leading_whitespace(tag("(")), tag(name)),
        leading_whitespace(inner),
        leading_whitespace(tag(")")),
    )
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes leading whitespace, returning the output of `inner`. This parser
/// also suppresses line comments.
pub fn leading_whitespace<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, O>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    preceded(preceded(multispace0, ignore_single_line_comment), inner)
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes leading and trailing whitespace, returning the output of `inner`.
/// Also suppresses line comments.
pub fn surrounding_whitespace<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, O>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    delimited(
        preceded(multispace0, ignore_single_line_comment),
        inner,
        preceded(multispace0, ignore_single_line_comment),
    )
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes a whitespace separated list, returning the outputs of `inner`.
pub fn space_separated_list0<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, Vec<O>>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    leading_whitespace(separated_list0(
        multispace1,
        preceded(ignore_single_line_comment, inner),
    ))
}

/// A combinator that takes a parser `inner` and produces a parser that also
/// consumes a whitespace separated list, returning the outputs of `inner`.
pub fn space_separated_list1<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, Vec<O>>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    leading_whitespace(separated_list1(
        multispace1,
        preceded(ignore_single_line_comment, inner),
    ))
}

/// A combinator that takes a parser `inner` and produces a parser that consumes
/// surrounding parentheses, returning the outputs of `inner`.
pub fn parens<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, O>
where
    F: FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    preceded(
        ignore_single_line_comment,
        delimited(char('('), leading_whitespace(inner), char(')')),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{parse_name, Match};
    use crate::Name;
    use nom::multi::separated_list1;

    #[test]
    fn parens_works() {
        let input = "(content)";
        let mut parser = parens(parse_name);
        assert!(parser(Span::new(input)).is_exactly("content"));
    }

    #[test]
    fn definition_section_works() {
        let input = "(either x y)";
        let inner_parser = separated_list1(tag(" "), parse_name);
        let mut parser = prefix_expr("either", inner_parser);
        assert!(parser(Span::new(input)).is_exactly(vec![Name::from("x"), Name::from("y")]));
    }

    #[test]
    fn space_separated_list0_works() {
        let mut parser = space_separated_list0(parse_name);
        assert!(parser(Span::new("x y")).is_exactly(vec![Name::from("x"), Name::from("y")]));
        assert!(parser(Span::new("x")).is_exactly(vec![Name::from("x")]));
        assert!(parser(Span::new("")).is_exactly(vec![]));
    }

    #[test]
    fn space_separated_list1_works() {
        let mut parser = space_separated_list1(parse_name);
        assert!(parser(Span::new("x y")).is_exactly(vec![Name::from("x"), Name::from("y")]));
        assert!(parser(Span::new("x")).is_exactly(vec![Name::from("x")]));
        assert!(parser(Span::new("")).is_err());
    }
}
