use crate::learning::{
    data_generators::DataGeneratorConfig, ml::MlModelConfig,
    models::preprocessor::PreprocessingOption, wl::WlConfig,
};
use serde::{Deserialize, Serialize};

// TODO-someday: the resulting config files are ugly and hard to read. Part of
// this is that toml doesn't seem enums nicely, but I've also just not put much
// effort into making the config files look nice.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WlModelConfig {
    pub model: MlModelConfig,
    pub wl: WlConfig,
    pub data_generator: DataGeneratorConfig,
    pub validate: bool,
    #[serde(default)]
    pub preprocessing_option: PreprocessingOption,
    /// Whether to tune the model, as of 2024/09/02, this is only supported for
    /// the LP ranker. If this is set to true, validate must also be set to
    /// true.
    #[serde(default = "default_tune")]
    pub tune: bool,
    // TODO-someday: this really should be a command line flag, not a part of
    // the model config, but this is convenient for now
    //
    // UPDATE: This might actually be fine/good? Changing to a "someday".
    #[serde(default = "default_explain_colours")]
    pub explain_colours: bool,
    /// Whether to round heuristic values to the nearest integer.
    pub round: bool,
    #[serde(default = "default_sparse_training_features")]
    pub sparse_training_features: bool,
}

fn default_tune() -> bool {
    false
}

fn default_explain_colours() -> bool {
    false
}

fn default_sparse_training_features() -> bool {
    false
}
