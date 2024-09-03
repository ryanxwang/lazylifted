use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, ColourDictionary, IlgCompiler},
        models::{RegressionTrainingData, TrainingData, TrainingInstance},
    },
    search::successor_generators::SuccessorGeneratorName,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateSpaceIlgRegressionConfig {
    pub successor_generator: SuccessorGeneratorName,
}

#[derive(Debug)]
pub struct StateSpaceIlgRegression {
    config: StateSpaceIlgRegressionConfig,
}

impl StateSpaceIlgRegression {
    pub fn new(config: &StateSpaceIlgRegressionConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl DataGenerator for StateSpaceIlgRegression {
    fn generate(
        &self,
        training_instances: &[TrainingInstance],
        colour_dictionary: &mut ColourDictionary,
    ) -> TrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut dist_to_goal = Vec::new();
        for instance in training_instances {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.config.successor_generator.create(task);
            let compiler = IlgCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            for (i, action) in plan.steps().iter().enumerate() {
                let action_schema = &task.action_schemas()[action.index];
                let next_state =
                    successor_generator.generate_successor(&cur_state, action_schema, action);
                graphs.push(compiler.compile(&cur_state, Some(colour_dictionary)));
                dist_to_goal.push(plan.len() as f64 - i as f64);
                cur_state = next_state;
            }
            graphs.push(compiler.compile(&cur_state, Some(colour_dictionary)));
            dist_to_goal.push(0.0);
        }

        assert_eq!(graphs.len(), dist_to_goal.len());
        TrainingData::Regression(RegressionTrainingData {
            features: graphs,
            labels: dist_to_goal,
            noise: None,
        })
    }
}
