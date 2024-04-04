use crate::parsed_types::Typed;
use std::ops::Deref;

/// A list of typed elements.
///
/// ## Example
/// ```
/// # use lazylifted::parsed_types::{Name, TypedList, Typed, Type};
/// let tl = TypedList::from_iter([
///     Typed::new(Name::from("location"), Type::OBJECT),
///     Typed::new(Name::from("physob"), Type::OBJECT),
/// ]);
///
/// assert_eq!(tl.len(), 2);
/// assert_eq!(tl[0].value(), &Name::from("location"));
/// assert_eq!(tl[1].value(), &Name::from("physob"));
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct TypedList<T>(Vec<Typed<T>>);

impl<T> TypedList<T> {
    pub const fn new(list: Vec<Typed<T>>) -> Self {
        Self(list)
    }

    /// Gets the values.
    pub fn value(&self) -> &[Typed<T>] {
        self.0.as_slice()
    }
}

impl<T> From<Vec<Typed<T>>> for TypedList<T> {
    fn from(iter: Vec<Typed<T>>) -> Self {
        TypedList::new(iter)
    }
}

impl<T> FromIterator<Typed<T>> for TypedList<T> {
    fn from_iter<I: IntoIterator<Item = Typed<T>>>(iter: I) -> Self {
        TypedList::new(iter.into_iter().collect())
    }
}

impl<T> Deref for TypedList<T> {
    type Target = [Typed<T>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<T> PartialEq<Vec<Typed<T>>> for TypedList<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Vec<Typed<T>>) -> bool {
        self.0.eq(other)
    }
}

impl<T> PartialEq<[Typed<T>]> for TypedList<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[Typed<T>]) -> bool {
        self.0.eq(other)
    }
}

impl<T> IntoIterator for TypedList<T> {
    type Item = Typed<T>;
    type IntoIter = std::vec::IntoIter<Typed<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
