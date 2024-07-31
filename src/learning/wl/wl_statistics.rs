use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone)]
pub struct WlStatistics {
    /// Sum of the sizes of all graphs seen so far
    total_graph_size: i64,
    /// Number of graphs seen so far
    num_graphs: i64,
    /// Number of hit colours
    num_hit_colours: i64,
    /// Number of miss colours
    num_miss_colours: i64,
    /// Time when the last log was printed, used for periodic logging
    last_log_time: Instant,
}

impl Default for WlStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl WlStatistics {
    pub fn new() -> Self {
        Self {
            total_graph_size: 0,
            num_graphs: 0,
            num_hit_colours: 0,
            num_miss_colours: 0,
            last_log_time: Instant::now(),
        }
    }

    pub fn register_graph(&mut self, graph_size: i64) {
        self.total_graph_size += graph_size;
        self.num_graphs += 1;
        self.log_if_needed();
    }

    // We do not log for hit/miss, as that we mean querying the time way too
    // often

    pub fn increment_hit_colours(&mut self) {
        self.num_hit_colours += 1;
    }

    pub fn increment_miss_colours(&mut self) {
        self.num_miss_colours += 1;
    }

    fn log_if_needed(&mut self) {
        if self.last_log_time.elapsed().as_secs() > 10 {
            self.last_log_time = Instant::now();
            self.log();
        }
    }

    fn log(&self) {
        info!(
            mean_graph_size = self.total_graph_size as f64 / self.num_graphs as f64,
            num_graphs = self.num_graphs,
            colour_miss_rate = self.num_miss_colours as f64
                / (self.num_hit_colours + self.num_miss_colours) as f64
        );
    }
}
