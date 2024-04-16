//! Contains action symbols via the [`ActionSymbol`] type.

use crate::parsed_types::Name;
use std::{fmt::Display, ops::Deref};

/// An action name.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct ActionName(Name);

impl ActionName {
    #[inline(always)]
    pub const fn new(name: Name) -> Self {
        Self(name)
    }

    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Self {
        Self(Name::new(name))
    }

    #[inline(always)]
    pub const fn from_name(name: Name) -> Self {
        Self(name)
    }
}

impl<T> From<T> for ActionName
where
    T: Into<Name>,
{
    #[inline(always)]
    fn from(value: T) -> Self {
        ActionName::new(value.into())
    }
}

impl AsRef<Name> for ActionName {
    #[inline(always)]
    fn as_ref(&self) -> &Name {
        &self.0
    }
}

impl AsRef<str> for ActionName {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for ActionName {
    type Target = Name;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ActionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
