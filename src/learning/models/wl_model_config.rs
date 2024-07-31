use serde::{Deserialize, Serialize};

use crate::learning::{
    data_generators::DataGeneratorConfig,
    ml::{MlModelName, RegressorName},
    wl::WlConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WlModelConfig {
    pub model: MlModelName,
    pub wl: WlConfig,
    pub data_generator: DataGeneratorConfig,
    pub validate: bool,
}

// This is an ugly way to allow for domain-custom model configurations :(
impl WlModelConfig {
    pub fn with_iters(self, iters: usize) -> Self {
        Self {
            wl: self.wl.with_iters(iters),
            ..self
        }
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
