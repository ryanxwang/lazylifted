use crate::{
    learning::{
        graphs::PartialActionCompilerName,
        ml::{MlModelName, RegressorName},
        wl::WlConfig,
    },
    search::successor_generators::SuccessorGeneratorName,
};
use serde::{Deserialize, Serialize};

/// Configuration for the partial action model. This is the format used by the
/// trainer to create the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialActionModelConfig {
    pub model: MlModelName,
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: PartialActionCompilerName,
    pub wl: WlConfig,
    pub validate: bool,
    pub group_partial_actions: bool,
}

impl PartialActionModelConfig {
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
