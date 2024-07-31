use crate::learning::{
    data_generators::{state_space_ilg_ranking::StateSpaceIlgRanking, DataGeneratorConfig},
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
        }
    }
}
