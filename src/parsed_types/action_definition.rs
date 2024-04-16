//! Contains action definitions via the [`ActionDefinition`] type.

use crate::parsed_types::TypedVariables;
use crate::parsed_types::{ActionName, PropCondition, PropEffect};

/// An action definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDefinition {
    name: ActionName,
    parameters: TypedVariables,
    preconditions: Vec<PropCondition>,
    effects: Vec<PropEffect>,
}

impl ActionDefinition {
    pub const fn new(
        name: ActionName,
        parameters: TypedVariables,
        preconditions: Vec<PropCondition>,
        effects: Vec<PropEffect>,
    ) -> Self {
        Self {
            name,
            parameters,
            preconditions,
            effects,
        }
    }

    pub const fn name(&self) -> &ActionName {
        &self.name
    }

    pub const fn parameters(&self) -> &TypedVariables {
        &self.parameters
    }

    pub const fn preconditions(&self) -> &Vec<PropCondition> {
        &self.preconditions
    }

    pub const fn effects(&self) -> &Vec<PropEffect> {
        &self.effects
    }
}

impl AsRef<ActionName> for ActionDefinition {
    fn as_ref(&self) -> &ActionName {
        &self.name
    }
}
