use crate::learning::data_generators::{
    state_space_ilg_ranking::StateSpaceIlgRankingConfig,
    state_space_ilg_regression::StateSpaceIlgRegressionConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataGeneratorConfig {
    StateSpaceIlgRanking(StateSpaceIlgRankingConfig),
    StateSpaceIlgRegression(StateSpaceIlgRegressionConfig),
}
