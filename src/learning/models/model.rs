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

    pub fn trainer_from_config(self) -> Box<dyn Train> {
        let py = unsafe { Python::assume_gil_acquired() };

        match self {
            ModelConfig::WlModel(config) => Box::new(WlModel::new(py, config)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        learning::{
            data_generators::DataGeneratorConfig,
            ml::MlModelConfig,
            wl::{SetOrMultiset, WlConfig},
        },
        search::successor_generators::SuccessorGeneratorName,
    };

    // This is not really a test, but more a helper piece of code to generate
    // serialised model configs
    #[test]
    fn serialise_sample_model_config() {
        let config = ModelConfig::WlModel(WlModelConfig {
            model: MlModelConfig::Ranker(crate::learning::ml::RankerConfig::LP { c_value: 0.1 }),
            wl: WlConfig {
                iters: 2,
                set_or_multiset: SetOrMultiset::Set,
            },
            data_generator: DataGeneratorConfig::StateSpaceRanking(
                crate::learning::data_generators::StateSpaceRankingConfig {
                    successor_generator: SuccessorGeneratorName::FullReducer,
                    graph_compiler: crate::learning::graphs::StateCompilerConfig::Ilg(
                        crate::learning::graphs::IlgConfig {
                            ignore_static_atoms: true,
                            use_edge_colours: true,
                        },
                    ),
                },
            ),
            preprocessing_option:
                crate::learning::models::preprocessor::PreprocessingOption::DivByStd,
            validate: true,
            tune: true,
            explain_colours: true,
            round: true,
            sparse_training_features: false,
        });

        let serialised = toml::to_string(&config).unwrap();
        println!("{}", serialised);
    }
}
