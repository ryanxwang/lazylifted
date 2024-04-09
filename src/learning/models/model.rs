//! This module contains traits and structs for training and evaluating models.

use crate::learning::models::{
    wl_aslg::{WLASLGConfig, WLASLGModel},
    wl_ilg::{WLILGConfig, WLILGModel},
};
use crate::search::{Plan, Task};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Evaluate {
    type EvaluatedType;

    /// Set the task that is currently being evaluated. After the first call to
    /// this method, further calls should be ignored.
    fn set_evaluating_task(&mut self, task: &Task);

    fn evaluate(&mut self, ts: &[&Self::EvaluatedType]) -> Vec<f64>;

    fn load(py: Python<'static>, path: &PathBuf) -> Self;
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

    fn save(&self, path: &PathBuf);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ModelConfig {
    #[serde(alias = "wl-ilg")]
    WLILG(WLILGConfig),
    #[serde(alias = "wl-aslg")]
    WLASLG(WLASLGConfig),
}

impl ModelConfig {
    pub fn load(path: &PathBuf) -> Box<dyn Train> {
        let py = unsafe { Python::assume_gil_acquired() };
        let config: ModelConfig = toml::from_str(
            &std::fs::read_to_string(path)
                .expect("Failed to read model config, does the file exist?"),
        )
        .expect("Failed to parse model config, is it valid?");

        match config {
            ModelConfig::WLILG(config) => Box::new(WLILGModel::new(py, config)),
            ModelConfig::WLASLG(config) => Box::new(WLASLGModel::new(py, config)),
        }
    }
}
