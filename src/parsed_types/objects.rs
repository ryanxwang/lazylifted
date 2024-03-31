//! Contains the [`Objects`] type.

use crate::parsed_types::{Name, Typed, TypedNames};
use std::ops::Deref;

/// A list of objects.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Objects(TypedNames);

impl Objects {
    pub fn new(objects: TypedNames) -> Self {
        Self(objects)
    }

    pub fn values(&self) -> &TypedNames {
        &self.0
    }
}

impl From<TypedNames> for Objects {
    fn from(value: TypedNames) -> Self {
        Self(value)
    }
}

impl FromIterator<Typed<Name>> for Objects {
    fn from_iter<T: IntoIterator<Item = Typed<Name>>>(iter: T) -> Self {
        Objects::new(TypedNames::from_iter(iter))
    }
}

impl Deref for Objects {
    type Target = TypedNames;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
