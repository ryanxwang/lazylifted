use tracing::info;

#[derive(Debug)]
pub struct SearchStatistics {
    /// Number of nodes expanded
    expanded_nodes: i32,
    /// Number of nodes evaluated
    evaluated_nodes: i32,
    /// Number of unique nodes generated
    generated_nodes: i32,
    /// Number of reopened nodes
    reopened_nodes: i32,
    /// Number of applicable actions generated
    generated_actions: i32,
    /// Number of preferred operator evaluations
    preferred_operator_evaluations: i32,
    /// Time when the search started
    search_start_time: std::time::Instant,
    /// Time when the last log was printed, used for periodic logging
    last_log_time: std::time::Instant,
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
            preferred_operator_evaluations: 0,
            search_start_time: std::time::Instant::now(),
            last_log_time: std::time::Instant::now(),
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
        self.generated_nodes += num_nodes as i32;
        self.log_if_needed();
    }

    pub fn increment_reopened_nodes(&mut self) {
        self.reopened_nodes += 1;
        self.log_if_needed();
    }

    pub fn increment_generated_actions(&mut self, num_actions: usize) {
        self.generated_actions += num_actions as i32;
        self.log_if_needed();
    }

    pub fn increment_preferred_operator_evaluations(&mut self) {
        self.preferred_operator_evaluations += 1;
        self.log_if_needed();
    }

    fn log_if_needed(&mut self) {
        if self.last_log_time.elapsed().as_secs() > 10 {
            self.log();
        }
    }

    pub fn log(&mut self) {
        self.last_log_time = std::time::Instant::now();
        info!(
            expanded_nodes = self.expanded_nodes,
            evaluated_nodes = self.evaluated_nodes,
            generated_nodes = self.generated_nodes,
            reopened_nodes = self.reopened_nodes,
            generated_actions = self.generated_actions,
            preferred_operator_evaluations = self.preferred_operator_evaluations
        );
    }

    pub fn finalise_search(&mut self) {
        info!("finalising search");
        self.log();
        info!(search_duration = self.search_start_time.elapsed().as_secs_f64());
    }
}
