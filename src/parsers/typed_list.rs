//! Provides the [`typed_list`] parser combinator.

use crate::parsed_types::{Typed, TypedList};
use crate::parsers::{
    leading_whitespace, parse_type, space_separated_list0, space_separated_list1, ParseResult, Span,
};
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{preceded, tuple};

/// Parser combinator that parses a typed list, i.e. `x* | x⁺ - \<type>` `<typed-list (x)>`.
///
/// ## Example
/// ```
/// # use nom::character::complete::alpha1;
/// # use lazylifted::parsers::{parse_name, typed_list, preamble::*};
/// # use lazylifted::parsed_types::*;
/// // Single implicitly typed element.
/// assert!(typed_list(parse_name)(Span::new("abc")).is_value(TypedList::from_iter([
///     Name::new("abc").to_typed(Type::OBJECT)
/// ])));
///
/// // Multiple implicitly typed elements.
/// assert!(typed_list(parse_name)(Span::new("abc def\nghi")).is_value(TypedList::from_iter([
///     Name::new("abc").to_typed(Type::OBJECT),
///     Name::new("def").to_typed(Type::OBJECT),
///     Name::new("ghi").to_typed(Type::OBJECT)
/// ])));
///
/// // Multiple explicitly typed elements.
/// assert!(typed_list(parse_name)(Span::new("abc def - word kitchen - room")).is_value(TypedList::from_iter([
///     Name::new("abc").to_typed("word"),
///     Name::new("def").to_typed("word"),
///     Name::new("kitchen").to_typed("room"),
/// ])));
///
/// // Mixed
/// assert!(typed_list(parse_name)(Span::new("abc def - word\ngeorgia - (either state country)\nuvw xyz")).is_value(TypedList::from_iter([
///     Name::new("abc").to_typed("word"),
///     Name::new("def").to_typed("word"),
///     Name::new("georgia").to_typed_either(["state", "country"]),
///     Name::new("uvw").to_typed(Type::OBJECT),
///     Name::new("xyz").to_typed(Type::OBJECT)
/// ])));
/// ```
pub fn typed_list<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> ParseResult<'a, TypedList<O>>
where
    F: Clone + FnMut(Span<'a>) -> ParseResult<'a, O>,
{
    // `x*`
    let implicitly_typed = map(inner.clone(), |o| Typed::new_object(o));
    let implicitly_typed_list = space_separated_list0(implicitly_typed);

    // `x⁺ - <type>`
    let explicitly_typed = map(
        tuple((
            space_separated_list1(inner.clone()),
            preceded(leading_whitespace(char('-')), parse_type),
        )),
        |(os, t)| {
            os.into_iter()
                .map(move |o| Typed::new(o, t.clone()))
                .collect::<Vec<_>>()
        },
    );

    let typed_list_choice = tuple((
        map(many0(explicitly_typed), |vec| {
            vec.into_iter().flatten().collect::<Vec<_>>()
        }),
        implicitly_typed_list,
    ));

    

    map(typed_list_choice, |(mut explicit, mut implicit)| {
        explicit.append(&mut implicit);
        TypedList::new(explicit)
    })
}
