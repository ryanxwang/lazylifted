use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, ColourDictionary, StateCompilerConfig},
        models::{RegressionTrainingData, TrainingData, TrainingInstance},
    },
    search::successor_generators::SuccessorGeneratorName,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateSpaceRegressionConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: StateCompilerConfig,
}

#[derive(Debug)]
pub struct StateSpaceRegression {
    config: StateSpaceRegressionConfig,
}

impl StateSpaceRegression {
    pub fn new(config: &StateSpaceRegressionConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl DataGenerator for StateSpaceRegression {
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
            let compiler = self
                .config
                .graph_compiler
                .create(task, self.config.successor_generator);

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
