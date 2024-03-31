//! Provides predicate definitions.

use crate::parsed_types::PredicateName;
use crate::parsed_types::{Name, TypedVariables};

/// Definition for a single predicate.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PredicateDefinition {
    predicate: PredicateName,
    variables: TypedVariables,
}

impl PredicateDefinition {
    pub const fn new(predicate: PredicateName, formula: TypedVariables) -> Self {
        Self {
            predicate,
            variables: formula,
        }
    }

    pub fn name(&self) -> &Name {
        self.predicate.as_ref()
    }

    /// Gets a reference to the predicate.
    pub const fn predicate(&self) -> &PredicateName {
        &self.predicate
    }

    /// Gets a reference to the variables.
    pub fn variables(&self) -> &TypedVariables {
        &self.variables
    }
}

impl From<(PredicateName, TypedVariables)> for PredicateDefinition {
    fn from(value: (PredicateName, TypedVariables)) -> Self {
        PredicateDefinition::new(value.0, value.1)
    }
}
