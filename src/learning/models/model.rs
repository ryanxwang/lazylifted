//! This module contains traits and structs for training and evaluating models.

use crate::learning::models::wl_ilg::{WLILGConfig, WLILGModel};
use crate::search::{Plan, Task};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Evaluate<'py> {
    type EvaluatedType;

    fn set_evaluating_task(&mut self, task: &Task);

    fn evaluate(&mut self, ts: &[Self::EvaluatedType]) -> Vec<f64>;

    fn load(py: Python<'py>, path: &PathBuf) -> Self;
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

pub trait Train<'py> {
    fn train(&mut self, py: Python<'py>, training_data: &[TrainingInstance]);

    // Save taking ownership of self is purely a skill issue, see
    // [`WLILGModel::save`] for an example of why.
    fn save(self, path: &PathBuf);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ModelConfig {
    #[serde(alias = "wl-ilg")]
    WLILG(WLILGConfig),
}

impl ModelConfig {
    pub fn load<'py>(py: Python<'py>, path: &PathBuf) -> impl Train<'py> {
        let config: ModelConfig = toml::from_str(
            &std::fs::read_to_string(path)
                .expect("Failed to read model config, does the file exist?"),
        )
        .expect("Failed to parse model config, is it valid?");

        match config {
            ModelConfig::WLILG(config) => WLILGModel::new(py, config),
        }
    }
}
