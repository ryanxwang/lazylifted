use crate::learning::{
    data_generators::{
        partial_space_ranking::PartialSpaceRanking,
        partial_space_regression::PartialSpaceRegression,
        partial_space_weighted_ranking::PartialSpaceWeightedRanking,
        state_space_ilg_ranking::StateSpaceIlgRanking,
        state_space_ilg_regression::StateSpaceIlgRegression, DataGeneratorConfig,
    },
    graphs::CGraph,
    models::{TrainingData, TrainingInstance},
};

pub trait DataGenerator {
    fn generate(&self, training_instances: &[TrainingInstance]) -> TrainingData<Vec<CGraph>>;
}

impl dyn DataGenerator {
    pub fn new(config: &DataGeneratorConfig) -> Box<dyn DataGenerator> {
        match config {
            DataGeneratorConfig::StateSpaceIlgRanking(config) => {
                Box::new(StateSpaceIlgRanking::new(config))
            }
            DataGeneratorConfig::StateSpaceIlgRegression(config) => {
                Box::new(StateSpaceIlgRegression::new(config))
            }
            DataGeneratorConfig::PartialSpaceRegression(config) => {
                Box::new(PartialSpaceRegression::new(config))
            }
            DataGeneratorConfig::PartialSpaceRanking(config) => {
                Box::new(PartialSpaceRanking::new(config))
            }
            DataGeneratorConfig::PartialSpaceWeightedRanking(config) => {
                Box::new(PartialSpaceWeightedRanking::new(config))
            }
        }
    }
}
