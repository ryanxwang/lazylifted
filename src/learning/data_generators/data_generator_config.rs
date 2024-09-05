use crate::learning::data_generators::{
    partial_space_dense_ranking::PartialSpaceDenseRankingConfig,
    partial_space_ranking::PartialSpaceRankingConfig,
    partial_space_regression::PartialSpaceRegressionConfig,
    state_space_ranking::StateSpaceRankingConfig,
    state_space_regression::StateSpaceRegressionConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataGeneratorConfig {
    StateSpaceRanking(StateSpaceRankingConfig),
    StateSpaceRegression(StateSpaceRegressionConfig),
    PartialSpaceRegression(PartialSpaceRegressionConfig),
    PartialSpaceRanking(PartialSpaceRankingConfig),
    PartialSpaceDenseRanking(PartialSpaceDenseRankingConfig),
}
