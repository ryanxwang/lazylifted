use crate::learning::data_generators::{
    partial_space_dense_ranking::PartialSpaceDenseRankingConfig,
    partial_space_ranking::PartialSpaceRankingConfig,
    partial_space_regression::PartialSpaceRegressionConfig,
    state_space_ilg_ranking::StateSpaceIlgRankingConfig,
    state_space_ilg_regression::StateSpaceIlgRegressionConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataGeneratorConfig {
    StateSpaceIlgRanking(StateSpaceIlgRankingConfig),
    StateSpaceIlgRegression(StateSpaceIlgRegressionConfig),
    PartialSpaceRegression(PartialSpaceRegressionConfig),
    PartialSpaceRanking(PartialSpaceRankingConfig),
    PartialSpaceDenseRanking(PartialSpaceDenseRankingConfig),
}
