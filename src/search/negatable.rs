use std::fmt::Display;

use strum_macros::EnumIs;

/// Wrapper around a type to indicate that it can be negated. Some types that
/// are often wrapped inside a [`Negatable`] are [`crate::search::Atom`] and
/// [`crate::search::AtomSchema`]. When using [`Negatable`] to wrap a type, it
/// is often very useful to implement some wrapper functions for
/// [`Negatable<T>`] as well, see [`crate::search::Atom`] for an example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIs)]
pub enum Negatable<T> {
    Positive(T),
    Negative(T),
}

impl<T> Negatable<T> {
    pub fn new(negated: bool, value: T) -> Self {
        if negated {
            Self::Negative(value)
        } else {
            Self::Positive(value)
        }
    }

    #[inline(always)]
    pub fn underlying(&self) -> &T {
        match self {
            Self::Positive(value) => value,
            Self::Negative(value) => value,
        }
    }
}

impl<T> From<T> for Negatable<T> {
    fn from(value: T) -> Self {
        Self::Positive(value)
    }
}

impl<T> Display for Negatable<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive(value) => write!(f, "{}", value),
            Self::Negative(value) => write!(f, "(not {})", value),
        }
    }
}
