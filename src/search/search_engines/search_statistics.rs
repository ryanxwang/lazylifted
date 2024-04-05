use crate::search::Verbosity;

pub struct SearchStatistics {
    /// Verbosity level
    pub verbosity: Verbosity,
    /// Number of nodes expanded
    pub expanded_nodes: i32,
    /// Number of nodes evaluated
    pub evaluated_nodes: i32,
    /// Number of unique nodes generated
    pub generated_nodes: i32,
    /// Number of reopened nodes
    pub reopened_nodes: i32,
    /// Number of applicable actions generated
    pub generated_actions: i32,
}

impl SearchStatistics {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            expanded_nodes: 0,
            evaluated_nodes: 0,
            generated_nodes: 0,
            reopened_nodes: 0,
            generated_actions: 0,
        }
    }

    pub fn increment_expanded_nodes(&mut self) {
        self.expanded_nodes += 1;
    }

    pub fn increment_evaluated_nodes(&mut self) {
        self.evaluated_nodes += 1;
    }

    pub fn increment_generated_nodes(&mut self) {
        self.generated_nodes += 1;
    }

    pub fn increment_reopened_nodes(&mut self) {
        self.reopened_nodes += 1;
    }

    pub fn increment_generated_actions(&mut self, num_actions: usize) {
        self.generated_actions += num_actions as i32;
    }
}
