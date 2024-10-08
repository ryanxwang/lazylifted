//! Provides type definitions;

use crate::parsed_types::{r#type::TYPE_OBJECT, Name, Type, Typed, TypedNames};
use std::ops::Deref;

/// A set of types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Types(TypedNames);

impl Types {
    pub fn new(mut typed_names: TypedNames) -> Self {
        // make sure that the object type is present
        let contains_object = typed_names
            .iter()
            .any(|typed| typed.value() == TYPE_OBJECT.name());
        if !contains_object {
            typed_names.push(Typed::new(TYPE_OBJECT.name().clone(), Type::OBJECT));
        }

        Self(typed_names)
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
