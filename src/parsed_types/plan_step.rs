//! Contains definitions for a single step of a plan via the [`PlanStep`] type.

use super::{ActionName, Name};

#[derive(Debug, Clone, PartialEq)]
pub struct PlanStep {
    name: ActionName,
    parameters: Vec<Name>,
}

impl PlanStep {
    pub const fn new(name: ActionName, parameters: Vec<Name>) -> Self {
        Self { name, parameters }
    }

    pub const fn name(&self) -> &ActionName {
        &self.name
    }

    pub const fn parameters(&self) -> &Vec<Name> {
        &self.parameters
    }
}
