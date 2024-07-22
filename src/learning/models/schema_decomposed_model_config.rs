use serde::{Deserialize, Serialize};

use crate::{learning::wl::WlConfig, search::successor_generators::SuccessorGeneratorName};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SchemaDecomposedModelConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub wl: WlConfig,
    pub validate: bool,
}

impl SchemaDecomposedModelConfig {
    pub fn with_iters(self, iters: usize) -> Self {
        Self {
            wl: self.wl.with_iters(iters),
            ..self
        }
    }
}
