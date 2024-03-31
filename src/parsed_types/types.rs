//! Provides type definitions;

use crate::{parsed_types::TypedNames, Name, Type, Typed};
use std::ops::Deref;

/// A set of types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Types(TypedNames);

impl Types {
    pub const fn new(predicates: TypedNames) -> Self {
        Self(predicates)
    }

    /// Gets the values.
    pub fn values(&self) -> &TypedNames {
        &self.0
    }
}

impl Default for Types {
    fn default() -> Self {
        Self::new(TypedNames::from_iter([Typed::new(
            Name::from("object"),
            Type::OBJECT,
        )]))
    }
}

impl Deref for Types {
    type Target = TypedNames;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TypedNames> for Types {
    fn from(value: TypedNames) -> Self {
        Types::new(value)
    }
}
