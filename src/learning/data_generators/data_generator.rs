use crate::learning::{
    data_generators::{
        partial_space_dense_ranking::PartialSpaceDenseRanking,
        partial_space_ranking::PartialSpaceRanking,
        partial_space_regression::PartialSpaceRegression, state_space_ranking::StateSpaceRanking,
        state_space_regression::StateSpaceRegression, DataGeneratorConfig,
    },
    graphs::{CGraph, ColourDictionary},
    models::{TrainingData, TrainingInstance},
};

pub trait DataGenerator {
    fn generate(
        &self,
        training_instances: &[TrainingInstance],
        colour_dictionary: &mut ColourDictionary,
    ) -> TrainingData<Vec<CGraph>>;
}

impl dyn DataGenerator {
    pub fn new(config: &DataGeneratorConfig) -> Box<dyn DataGenerator> {
        match config {
            DataGeneratorConfig::StateSpaceRanking(config) => {
                Box::new(StateSpaceRanking::new(config))
            }
            DataGeneratorConfig::StateSpaceRegression(config) => {
                Box::new(StateSpaceRegression::new(config))
            }
            DataGeneratorConfig::PartialSpaceRegression(config) => {
                Box::new(PartialSpaceRegression::new(config))
            }
            DataGeneratorConfig::PartialSpaceRanking(config) => {
                Box::new(PartialSpaceRanking::new(config))
            }
            DataGeneratorConfig::PartialSpaceDenseRanking(config) => {
                Box::new(PartialSpaceDenseRanking::new(config))
            }
        }
    }
}
