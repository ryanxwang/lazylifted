//! Contains definitions for a plan via the [`Plan`] type.

use crate::parsed_types::PlanStep;

#[derive(Debug, Clone, PartialEq)]
pub struct Plan(Vec<PlanStep>);

impl Plan {
    pub const fn new(steps: Vec<PlanStep>) -> Self {
        Self(steps)
    }

    pub const fn steps(&self) -> &Vec<PlanStep> {
        &self.0
    }
}
