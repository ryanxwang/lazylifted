//! This module contains traits and structs for training and evaluating models.

use crate::learning::models::{
    partial_action_model::PartialActionModel,
    partial_action_model_config::PartialActionModelConfig,
    state_space_model_config::StateSpaceModelConfig, StateSpaceModel,
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

    // fn evaluate_batch(&mut self, ts: &[Self::EvaluatedType<'_>]) -> Vec<f64>;

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
#[serde(rename_all = "kebab-case")]
pub enum ModelConfig {
    StateSpaceModel(StateSpaceModelConfig),
    PartialActionModel(PartialActionModelConfig),
}

impl ModelConfig {
    pub fn from_path(path: &Path) -> Self {
        let config: ModelConfig = toml::from_str(
            &std::fs::read_to_string(path)
                .expect("Failed to read model config, does the file exist?"),
        )
        .expect("Failed to parse model config, is it valid?");
        config
    }

    pub fn trainer_from_config(self, iters: Option<usize>, alpha: Option<f64>) -> Box<dyn Train> {
        let py = unsafe { Python::assume_gil_acquired() };

        match self {
            ModelConfig::StateSpaceModel(config) => {
                let config = if let Some(iters) = iters {
                    config.with_iters(iters)
                } else {
                    config
                };
                let config = if let Some(alpha) = alpha {
                    config.with_alpha(alpha)
                } else {
                    config
                };
                Box::new(StateSpaceModel::new(py, config))
            }
            ModelConfig::PartialActionModel(config) => {
                let config = if let Some(iters) = iters {
                    config.with_iters(iters)
                } else {
                    config
                };
                let config = if let Some(alpha) = alpha {
                    config.with_alpha(alpha)
                } else {
                    config
                };
                Box::new(PartialActionModel::new(py, config))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        learning::{
            graphs::PartialActionCompilerName,
            ml::MlModelName,
            models::partial_action_model_config::PartialActionModelConfig,
            wl::{SetOrMultiset, WlConfig},
        },
        search::successor_generators::SuccessorGeneratorName,
    };

    // This is not really a test, but more a helper piece of code to make
    // serialised model configs
    #[test]
    fn serialise_sample_model_config() {
        let config = ModelConfig::PartialActionModel(PartialActionModelConfig {
            model: MlModelName::RankerName(crate::learning::ml::RankerName::LP),
            graph_compiler: PartialActionCompilerName::Rslg,
            wl: WlConfig {
                iters: 1,
                set_or_multiset: SetOrMultiset::Multiset,
            },
            validate: false,
            successor_generator: SuccessorGeneratorName::FullReducer,
        });

        let serialised = toml::to_string(&config).unwrap();
        println!("{}", serialised);
    }
}
