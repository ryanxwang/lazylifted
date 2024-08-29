use crate::learning::{
    data_generators::DataGeneratorConfig, ml::MlModelConfig,
    models::preprocessor::PreprocessingOption, wl::WlConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WlModelConfig {
    pub model: MlModelConfig,
    pub wl: WlConfig,
    pub data_generator: DataGeneratorConfig,
    pub validate: bool,
    #[serde(default)]
    pub preprocessing_option: PreprocessingOption,
}
