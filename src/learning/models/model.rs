//! This module contains traits and structs for training and evaluating models.

use crate::learning::models::{
    wl_ilg::{WlIlgConfig, WlIlgModel},
    wl_palg::{WlPalgConfig, WlPalgModel},
};
use crate::search::{Plan, Task};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait Evaluate {
    type EvaluatedType<'a>;

    /// Set the task that is currently being evaluated. After the first call to
    /// this method, further calls should be ignored.
    fn set_evaluating_task(&mut self, task: &Task);

    fn evaluate(&mut self, t: &Self::EvaluatedType<'_>) -> f64;

    fn evaluate_batch(&mut self, ts: &[Self::EvaluatedType<'_>]) -> Vec<f64>;

    fn load(py: Python<'static>, path: &Path) -> Self;
}

/// A training instance is a pair of a plan and a task.
#[derive(Debug)]
pub struct TrainingInstance {
    pub plan: Plan,
    pub task: Task,
}

impl TrainingInstance {
    pub fn new(plan: Plan, task: Task) -> Self {
        Self { plan, task }
    }
}

pub trait Train {
    fn train(&mut self, training_data: &[TrainingInstance]);

    fn save(&self, path: &Path);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ModelConfig {
    #[serde(alias = "wl-ilg")]
    WLILG(WlIlgConfig),
    #[serde(alias = "wl-palg")]
    WLPALG(WlPalgConfig),
}

impl ModelConfig {
    pub fn load(path: &Path) -> Box<dyn Train> {
        let py = unsafe { Python::assume_gil_acquired() };
        let config: ModelConfig = toml::from_str(
            &std::fs::read_to_string(path)
                .expect("Failed to read model config, does the file exist?"),
        )
        .expect("Failed to parse model config, is it valid?");

        match config {
            ModelConfig::WLILG(config) => Box::new(WlIlgModel::new(py, config)),
            ModelConfig::WLPALG(config) => Box::new(WlPalgModel::new(py, config)),
        }
    }
}
