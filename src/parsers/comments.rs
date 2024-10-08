use crate::parsers::{ParseResult, Span};
use nom::bytes::complete::is_not;
use nom::character::complete::{char, multispace0};
use nom::combinator::{opt, value};
use nom::sequence::{pair, terminated, tuple};

pub fn ignore_single_line_comment<'a, S: Into<Span<'a>>>(input: S) -> ParseResult<'a, ()> {
    value(
        (),
        opt(terminated(
            pair(char(';'), opt(is_not("\r\n"))),
            tuple((multispace0, opt(ignore_single_line_comment))),
        )),
    )(input.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_semicolon() {
        let input = ";\n";
        let (remainder, _comment) = ignore_single_line_comment(input).unwrap();
        println!("{:?}", remainder);
        assert!(remainder.is_empty());
    }

    #[test]
    fn comment_only() {
        let input = "; comment";
        let (remainder, _comment) = ignore_single_line_comment(input).unwrap();
        assert!(remainder.is_empty());
    }

    #[test]
    fn keeps_text() {
        let input = "; comment\nnext line";
        let (remainder, _comment) = ignore_single_line_comment(input).unwrap();
        assert_eq!(remainder.fragment(), &"next line");
    }
}
