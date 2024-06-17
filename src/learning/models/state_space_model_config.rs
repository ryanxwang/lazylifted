use crate::{
    learning::ml::{MlModelName, RegressorName},
    search::successor_generators::SuccessorGeneratorName,
};
use serde::{Deserialize, Serialize};

/// Configuration for the WL-ILG model. This is the format used by the trainer
/// to create the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateSpaceModelConfig {
    pub model: MlModelName,
    pub successor_generator: SuccessorGeneratorName,
    pub iters: usize,
    pub validate: bool,
}

impl StateSpaceModelConfig {
    pub fn with_iters(self, iters: usize) -> Self {
        Self { iters, ..self }
    }

    pub fn with_alpha(self, alpha: f64) -> Self {
        Self {
            model: match self.model {
                MlModelName::RegressorName(RegressorName::GaussianProcessRegressor { .. }) => {
                    MlModelName::RegressorName(RegressorName::GaussianProcessRegressor { alpha })
                }
                _ => self.model,
            },
            ..self
        }
    }
}
