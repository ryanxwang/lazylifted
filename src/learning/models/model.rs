//! This module contains traits and structs for training and evaluating models.

use crate::learning::models::{wl_model_config::WlModelConfig, WlModel};
use crate::search::{Plan, Task};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait Evaluate {
    type EvaluatedType<'a>;

    fn evaluate(&mut self, t: Self::EvaluatedType<'_>, group_id: Option<usize>) -> f64;

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
    WlModel(WlModelConfig),
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
            ModelConfig::WlModel(config) => {
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
                Box::new(WlModel::new(py, config))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        learning::{
            data_generators::DataGeneratorConfig,
            ml::MlModelName,
            wl::{SetOrMultiset, WlConfig},
        },
        search::successor_generators::SuccessorGeneratorName,
    };

    // This is not really a test, but more a helper piece of code to make
    // serialised model configs
    #[test]
    fn serialise_sample_model_config() {
        let config = ModelConfig::WlModel(WlModelConfig {
            model: MlModelName::RankerName(crate::learning::ml::RankerName::LP),
            wl: WlConfig {
                iters: 2,
                set_or_multiset: SetOrMultiset::Set,
            },
            validate: true,
            data_generator: DataGeneratorConfig::StateSpaceIlgRanking(
                crate::learning::data_generators::StateSpaceIlgRankingConfig {
                    successor_generator: SuccessorGeneratorName::FullReducer,
                },
            ),
            preprocessing_option:
                crate::learning::models::preprocessor::PreprocessingOption::DivByStd,
        });

        let serialised = toml::to_string(&config).unwrap();
        println!("{}", serialised);
    }
}
