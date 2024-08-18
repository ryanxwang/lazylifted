use crate::search::HeuristicValue;
use ordered_float::Float;
use std::time::Instant;
use tracing::info;

#[derive(Debug)]
pub struct SearchStatistics {
    /// Number of nodes expanded
    expanded_nodes: i64,
    /// Number of nodes evaluated
    evaluated_nodes: i64,
    /// Number of unique nodes generated
    generated_nodes: i64,
    /// Number of reopened nodes
    reopened_nodes: i64,
    /// Number of applicable actions generated
    generated_actions: i64,
    /// Number of evaluations skipped as there is only one applicable transition
    skipped_evaluations: i64,
    /// Best heuristic value found so far
    best_heuristic_value: HeuristicValue,
    /// Time when the search started
    search_start_time: Instant,
    /// Time when the last log was printed, used for periodic logging
    last_log_time: Instant,
}

impl Default for SearchStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchStatistics {
    pub fn new() -> Self {
        info!("starting search");
        Self {
            expanded_nodes: 0,
            evaluated_nodes: 0,
            generated_nodes: 0,
            reopened_nodes: 0,
            generated_actions: 0,
            skipped_evaluations: 0,
            best_heuristic_value: HeuristicValue::infinity(),
            search_start_time: Instant::now(),
            last_log_time: Instant::now(),
        }
    }

    pub fn register_heuristic_value(&mut self, heuristic_value: HeuristicValue) {
        if heuristic_value < self.best_heuristic_value {
            self.best_heuristic_value = heuristic_value;
            info!(best_heuristic_value = self.best_heuristic_value.into_inner());
            self.last_log_time = Instant::now();
            self.log();
        }
    }

    pub fn increment_expanded_nodes(&mut self) {
        self.expanded_nodes += 1;
        self.log_if_needed();
    }

    pub fn increment_evaluated_nodes(&mut self) {
        self.evaluated_nodes += 1;
        self.log_if_needed();
    }

    pub fn increment_generated_nodes(&mut self, num_nodes: usize) {
        self.generated_nodes += num_nodes as i64;
        self.log_if_needed();
    }

    pub fn increment_reopened_nodes(&mut self) {
        self.reopened_nodes += 1;
        self.log_if_needed();
    }

    pub fn increment_generated_actions(&mut self, num_actions: usize) {
        self.generated_actions += num_actions as i64;
        self.log_if_needed();
    }

    pub fn increment_skipped_evaluations(&mut self) {
        self.skipped_evaluations += 1;
        self.log_if_needed();
    }

    fn log_if_needed(&mut self) {
        if self.last_log_time.elapsed().as_secs() > 10 {
            self.last_log_time = Instant::now();
            self.log();
        }
    }

    fn log(&self) {
        // TODO(someday): would be nice to log memory usage, like FD does.
        // https://crates.io/crates/memory-stats seems like a decent tool for
        // this.
        info!(
            expanded_nodes = self.expanded_nodes,
            evaluated_nodes = self.evaluated_nodes,
            generated_nodes = self.generated_nodes,
            reopened_nodes = self.reopened_nodes,
            generated_actions = self.generated_actions,
            skipped_evaluations = self.skipped_evaluations,
            best_heuristic_value = self.best_heuristic_value.into_inner(),
        );
    }

    pub fn finalise_search(&self) {
        info!("finalising search");
        self.log();
        info!(search_duration = self.search_start_time.elapsed().as_secs_f64());
    }
}
