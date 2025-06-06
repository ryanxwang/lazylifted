//! Contains typed elements.

use crate::parsed_types::{PrimitiveType, Type};
use std::ops::Deref;

/// A typed element.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Typed<O>(O, Type);

impl<O> Typed<O> {
    pub const fn new(value: O, r#type: Type) -> Self {
        Self(value, r#type)
    }

    pub const fn new_object(value: O) -> Self {
        Self::new(value, Type::OBJECT)
    }

    /// Gets the value.
    pub const fn value(&self) -> &O {
        &self.0
    }

    /// Gets the assigned type.
    pub const fn type_(&self) -> &Type {
        &self.1
    }
}

pub trait ToTyped<T> {
    /// Wraps the value into a [`Typed`] as [`Type::Exactly`] the specified type.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsed_types::{Name, PrimitiveType, ToTyped, Type, Typed};
    /// assert_eq!(
    ///     Name::from("kitchen").to_typed("room"),
    ///     Typed::new(Name::from("kitchen"), Type::Exactly(PrimitiveType::from("room")))
    /// );
    /// ```
    fn to_typed<I: Into<Type>>(self, r#type: I) -> Typed<T>;

    /// Wraps the value into a [`Typed`] as [`Type::EitherOf`] the specified types.
    ///
    /// ## Example
    /// ```
    /// # use lazylifted::parsed_types::{Name, PrimitiveType, ToTyped, Type, Typed};
    /// assert_eq!(
    ///     Name::from("georgia").to_typed_either(["country", "state"]),
    ///     Typed::new(Name::from("georgia"), Type::EitherOf(
    ///         vec![
    ///             PrimitiveType::from("country"),
    ///             PrimitiveType::from("state")
    ///         ])
    ///     )
    /// );
    /// ```
    fn to_typed_either<I: IntoIterator<Item = P>, P: Into<PrimitiveType>>(
        self,
        r#type: I,
    ) -> Typed<T>;
}

impl<O> From<O> for Typed<O> {
    fn from(value: O) -> Self {
        Typed::new_object(value)
    }
}

impl<O> Deref for Typed<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}
