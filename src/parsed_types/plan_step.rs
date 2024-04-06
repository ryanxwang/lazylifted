//! Contains definitions for a single step of a plan via the [`PlanStep`] type.

use super::{ActionName, Name};

#[derive(Debug, Clone, PartialEq)]
pub struct PlanStep {
    action_name: ActionName,
    parameters: Vec<Name>,
}

impl PlanStep {
    pub const fn new(action_name: ActionName, parameters: Vec<Name>) -> Self {
        Self {
            action_name: action_name,
            parameters,
        }
    }

    pub const fn action_name(&self) -> &ActionName {
        &self.action_name
    }

    pub const fn parameters(&self) -> &Vec<Name> {
        &self.parameters
    }
}
