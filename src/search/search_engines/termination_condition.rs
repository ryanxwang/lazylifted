use crate::search::search_engines::SearchResult;
use memory_stats::memory_stats;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug)]
pub struct TerminationCondition {
    time_limit: Option<Duration>,
    memory_limit_mb: Option<usize>,
    start_time: Instant,
    peak_memory_usage_mb: Option<usize>,
    last_log_time: Instant,
}

impl TerminationCondition {
    pub fn new(time_limit: Option<Duration>, memory_limit_mb: Option<usize>) -> Self {
        info!(
            time_limit = time_limit.map(|d| d.as_secs_f64()),
            memory_limit_mb = memory_limit_mb,
        );
        Self {
            time_limit,
            memory_limit_mb,
            start_time: Instant::now(),
            peak_memory_usage_mb: None,
            last_log_time: Instant::now(),
        }
    }

    pub fn log_if_needed(&mut self) {
        if self.last_log_time.elapsed() > Duration::from_secs(10) {
            self.last_log_time = Instant::now();
            self.log();
        }
    }

    pub fn log(&mut self) {
        let memory_usage = memory_stats().map(|usage| usage.physical_mem / 1024 / 1024);
        self.peak_memory_usage_mb = self.peak_memory_usage_mb.max(memory_usage);
        let time_elapsed = self.start_time.elapsed();
        info!(
            memory_usage_mb = memory_usage,
            time_elapsed = time_elapsed.as_secs_f64(),
        );
    }

    pub fn finalise(&mut self) {
        let time_elapsed = self.start_time.elapsed();
        info!(
            peak_recorded_memory_usage_mb = self.peak_memory_usage_mb,
            total_time_used = time_elapsed.as_secs_f64(),
        );
    }

    pub fn should_terminate(&self) -> Option<SearchResult> {
        if let Some(time_limit) = self.time_limit {
            if self.start_time.elapsed() > time_limit {
                return Some(SearchResult::TimeLimitExceeded);
            }
        }
        if let Some(memory_limit_mb) = self.memory_limit_mb {
            if let Some(peak_usage) = self.peak_memory_usage_mb {
                if peak_usage > memory_limit_mb {
                    return Some(SearchResult::MemoryLimitExceeded);
                }
            }
        }
        None
    }
}
