use crate::{
    learning::{
        graphs::{CGraph, IlgCompiler},
        ml::MlModel,
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            state_space_model_config::StateSpaceModelConfig,
            Evaluate, Train, TrainingInstance,
        },
        WlKernel,
    },
    search::{successor_generators::SuccessorGeneratorName, Action, DBState, Task},
};
use numpy::PyUntypedArrayMethods;
use pyo3::{types::PyAnyMethods, Python};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, time};
use tempfile::NamedTempFile;
use tracing::info;

use super::{
    RankingPair, RankingRelation, RankingTrainingData, RegressionTrainingData, TrainingData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ModelState {
    /// The model has been created but not trained.
    New,
    /// Trained but not ready for evaluating.
    Trained,
    /// Ready for evaluating.
    Evaluating(IlgCompiler),
}

impl PartialEq for ModelState {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ModelState::New, ModelState::New)
                | (ModelState::Trained, ModelState::Trained)
                | (ModelState::Evaluating(_), ModelState::Evaluating(_))
        )
    }
}

#[derive(Debug)]
pub struct StateSpaceModel {
    model: MlModel<'static>,
    /// The successor generator to use for generating successor states when
    /// training. It might appear weird we store the name of the successor
    /// generator instead of the generator itself, but this is because 1)
    /// it is only used in training, and 2) each task requires its own successor
    /// generator, so we can't store a single instance of the generator.
    successor_generator_name: SuccessorGeneratorName,
    wl: WlKernel,
    validate: bool,
    state: ModelState,
    config: StateSpaceModelConfig,
}

/// Dummy struct to allow serialising/deserialising the model to disk.
#[derive(Debug, Serialize, Deserialize)]
struct SerialisableStateSpaceModel {
    successor_generator_name: SuccessorGeneratorName,
    wl: WlKernel,
    validate: bool,
    state: ModelState,
    config: StateSpaceModelConfig,
}

impl StateSpaceModel {
    pub fn new(py: Python<'static>, config: StateSpaceModelConfig) -> Self {
        Self {
            model: MlModel::new(py, config.model),
            wl: WlKernel::new(config.iters),
            successor_generator_name: config.successor_generator,
            validate: config.validate,
            state: ModelState::New,
            config: config.clone(),
        }
    }

    fn prepare_ranking_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> RankingTrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();

        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = IlgCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            let mut predecessor_graph: Option<CGraph> = None;
            let mut sibling_graphs: Option<Vec<CGraph>> = None;
            for chosen_action in plan.steps() {
                let cur_graph = compiler.compile(&cur_state);
                let cur_index = graphs.len();
                graphs.push(cur_graph.clone());

                // First rank this state better than its predecessors
                if let Some(predecessor_graph) = &predecessor_graph {
                    pairs.push(RankingPair {
                        i: cur_index,
                        j: graphs.len(),
                        relation: RankingRelation::Better,
                    });
                    graphs.push(predecessor_graph.clone());
                }

                // Then rank it better than or equal to its siblings
                if let Some(sibling_graphs) = &sibling_graphs {
                    for sibling_graph in sibling_graphs {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: graphs.len(),
                            relation: RankingRelation::BetterOrEqual,
                        });
                        graphs.push(sibling_graph.clone());
                    }
                }

                // Update the structs
                sibling_graphs = Some(vec![]);
                let applicable_actions: Vec<Action> = task
                    .action_schemas()
                    .iter()
                    .flat_map(|schema| {
                        successor_generator.get_applicable_actions(&cur_state, schema)
                    })
                    .collect();
                for action in applicable_actions {
                    if action == *chosen_action {
                        continue;
                    }

                    let action_schema = &task.action_schemas()[action.index];
                    let next_state =
                        successor_generator.generate_successor(&cur_state, action_schema, &action);
                    let next_graph = compiler.compile(&next_state);
                    sibling_graphs.as_mut().unwrap().push(next_graph);
                }

                predecessor_graph = Some(cur_graph);

                cur_state = successor_generator.generate_successor(
                    &cur_state,
                    &task.action_schemas()[chosen_action.index],
                    chosen_action,
                );
            }
        }

        RankingTrainingData {
            features: graphs,
            pairs,
        }
    }

    fn prepare_regression_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> RegressionTrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut dist_to_goal = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = IlgCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            for (i, action) in plan.steps().iter().enumerate() {
                let action_schema = &task.action_schemas()[action.index];
                let next_state =
                    successor_generator.generate_successor(&cur_state, action_schema, action);
                graphs.push(compiler.compile(&cur_state));
                dist_to_goal.push(plan.len() as f64 - i as f64);
                cur_state = next_state;
            }
            graphs.push(compiler.compile(&cur_state));
            dist_to_goal.push(0.0);
        }

        assert_eq!(graphs.len(), dist_to_goal.len());
        RegressionTrainingData {
            features: graphs,
            labels: dist_to_goal,
            noise: None,
        }
    }

    fn prepare_data(&self, training_data: &[TrainingInstance]) -> TrainingData<Vec<CGraph>> {
        match self.model {
            MlModel::Regressor(_) => {
                TrainingData::Regression(self.prepare_regression_data(training_data))
            }
            MlModel::Ranker(_) => TrainingData::Ranking(self.prepare_ranking_data(training_data)),
        }
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for StateSpaceModel {
    fn train(&mut self, train_instances: &[TrainingInstance]) {
        let py = self.py();
        assert_eq!(self.state, ModelState::New);
        if self.validate {
            info!("splitting training data into training and validation sets");
        } else {
            info!("training on full dataset");
        }
        let (train_instances, val_instances) = match self.validate {
            true => train_instances.split_at((train_instances.len() as f64 * 0.8) as usize),
            // Without this trivial cast we get a dumb error message
            #[allow(trivial_casts)]
            false => (train_instances, &[] as &[TrainingInstance]),
        };

        let train_data = self.prepare_data(train_instances);
        let train_graphs = &train_data.features();
        let mean_train_graph_size = train_graphs.iter().map(|g| g.node_count()).sum::<usize>()
            as f64
            / train_graphs.len() as f64;
        let val_data = self.prepare_data(val_instances);
        let val_graphs = &val_data.features();
        info!("compiled states into graphs");
        info!(
            train_graphs = train_graphs.len(),
            mean_train_graph_size = mean_train_graph_size,
            val_graphs = val_graphs.len()
        );

        let train_histograms = self.wl.compute_histograms(train_graphs);
        let val_histograms = self.wl.compute_histograms(val_graphs);
        info!("computed WL histograms");

        let train_x = self.wl.compute_x(py, &train_histograms);
        let val_x = self.wl.compute_x(py, &val_histograms);
        info!("computed WL features");
        self.wl.finalise();

        info!(
            train_x_shape = format!("{:?}", train_x.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
        );
        let train_data = train_data.with_features(train_x);
        let val_data = val_data.with_features(val_x);

        info!("fitting model on training data");
        self.model.fit(&train_data);

        let train_score_start = time::Instant::now();
        let train_score = self.model.score(&train_data);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        info!(train_score = train_score);
        if self.validate {
            let val_score_start = time::Instant::now();
            let val_score = self.model.score(&val_data);
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            info!(val_score = val_score);
        }

        self.state = ModelState::Trained;
    }

    fn save(&self, path: &Path) {
        assert_eq!(self.state, ModelState::Trained);

        let pickle_file = NamedTempFile::new().expect("Failed to create temporary file");
        let mut ron_file = NamedTempFile::new().expect("Failed to create temporary file");

        self.model.pickle(pickle_file.path());

        let serialisable = SerialisableStateSpaceModel {
            successor_generator_name: self.successor_generator_name,
            wl: self.wl.clone(),
            validate: self.validate,
            state: self.state.clone(),
            config: self.config.clone(),
        };
        let serialised = ron::to_string(&serialisable).expect("Failed to serialise model data");

        ron_file
            .write_all(serialised.as_bytes())
            .expect("Failed to write model data");

        zip_files(
            path,
            vec![
                (PICKLE_FILE_NAME, pickle_file.path()),
                (RON_FILE_NAME, ron_file.path()),
            ],
        );
        info!("saved model to {}", path.display());
    }
}

impl Evaluate for StateSpaceModel {
    type EvaluatedType<'a> = DBState;

    fn set_evaluating_task(&mut self, task: &Task) {
        match &self.state {
            ModelState::New => {
                panic!("Model not trained yet, cannot set evaluating task");
            }
            ModelState::Trained => self.state = ModelState::Evaluating(IlgCompiler::new(task)),
            ModelState::Evaluating(_) => {}
        }
    }

    fn evaluate(&mut self, state: &DBState) -> f64 {
        let compiler = match &self.state {
            ModelState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graph = compiler.compile(state);
        let histograms = self.wl.compute_histograms(&[graph]);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y = self.model.predict(&x);
        let y: Vec<f64> = y.extract().unwrap();
        y[0]
    }

    fn evaluate_batch(&mut self, states: &[DBState]) -> Vec<f64> {
        let compiler = match &self.state {
            ModelState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        // when evaluating in batch, we still do it sequentially for better
        // cache locality
        let graphs = states
            .iter()
            .map(|t| compiler.compile(t))
            .collect::<Vec<_>>();
        let histograms = self.wl.compute_histograms(&graphs);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y = self.model.predict(&x);
        y.extract().unwrap()
    }

    fn load(py: Python<'static>, path: &Path) -> Self {
        let ron_file = extract_from_zip(path, RON_FILE_NAME);
        let data = std::fs::read_to_string(ron_file).expect("Failed to read model data");
        let serialisable: SerialisableStateSpaceModel =
            ron::from_str(&data).expect("Failed to deserialise model data");
        assert_eq!(serialisable.state, ModelState::Trained);

        let pickle_file = extract_from_zip(path, PICKLE_FILE_NAME);
        let model = MlModel::unpickle(serialisable.config.model, py, pickle_file.path());

        Self {
            model,
            successor_generator_name: serialisable.successor_generator_name,
            wl: serialisable.wl,
            validate: serialisable.validate,
            state: serialisable.state,
            config: serialisable.config,
        }
    }
}
