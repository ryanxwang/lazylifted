use crate::{
    learning::{
        data_generators::DataGenerator,
        graphs::{CGraph, PartialActionCompilerName},
        models::{RegressionTrainingData, TrainingData, TrainingInstance},
    },
    search::{successor_generators::SuccessorGeneratorName, PartialAction},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialSpaceRegressionConfig {
    pub successor_generator: SuccessorGeneratorName,
    pub graph_compiler: PartialActionCompilerName,
}

#[derive(Debug)]
pub struct PartialSpaceRegression {
    config: PartialSpaceRegressionConfig,
}

impl PartialSpaceRegression {
    pub fn new(config: &PartialSpaceRegressionConfig) -> Self {
        PartialSpaceRegression {
            config: config.clone(),
        }
    }
}

impl DataGenerator for PartialSpaceRegression {
    fn generate(&self, training_instances: &[TrainingInstance]) -> TrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut labels = Vec::new();
        let mut noise = Vec::new();
        for instance in training_instances {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.config.successor_generator.create(task);
            let compiler = self
                .config
                .graph_compiler
                .create(task, self.config.successor_generator);

            let total_steps = plan.steps().len() as f64;

            let mut cur_state = task.initial_state.clone();
            let mut cur_partial_step = 0.;
            for chosen_action in plan.steps() {
                for partial_depth in 0..=(chosen_action.instantiation.len()) {
                    cur_partial_step += 1. / (chosen_action.instantiation.len() + 1) as f64;
                    let partial = PartialAction::from_action(chosen_action, partial_depth);
                    graphs.push(compiler.compile(&cur_state, &partial));
                    labels.push(total_steps - cur_partial_step);

                    if partial_depth == chosen_action.instantiation.len() {
                        noise.push(0.0);
                    } else {
                        // we hardcode this value for now, but what should it
                        // actually be?
                        //
                        // Update: for now ranking is clearly doing better than
                        // regression, so this value doesn't really matter
                        noise.push(0.3);
                    }
                }

                cur_state = successor_generator.generate_successor(
                    &cur_state,
                    &task.action_schemas()[chosen_action.index],
                    chosen_action,
                );
            }
        }

        TrainingData::Regression(RegressionTrainingData {
            features: graphs,
            labels,
            noise: Some(noise),
        })
    }
}
