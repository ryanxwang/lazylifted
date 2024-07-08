use crate::{
    learning::{
        graphs::{CGraph, PartialActionCompiler, PartialActionCompilerName},
        ml::MlModel,
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            partial_action_model_config::PartialActionModelConfig,
            Evaluate, RankingPair, RankingRelation, RankingTrainingData, RegressionTrainingData,
            Train, TrainingData, TrainingInstance,
        },
        wl::{Neighbourhood, WlKernel},
    },
    search::{successor_generators::SuccessorGeneratorName, Action, DBState, PartialAction, Task},
};
use numpy::PyUntypedArrayMethods;
use pyo3::{prelude::*, Python};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Write, path::Path};
use tempfile::NamedTempFile;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
enum PartialActionModelState {
    // The model has been created but not trained
    New,
    // Trained but not ready for evaluating
    Trained,
    // Ready for evaluating
    #[serde(skip)]
    Evaluating(Box<dyn PartialActionCompiler>),
}

impl PartialEq for PartialActionModelState {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (PartialActionModelState::New, PartialActionModelState::New)
                | (
                    PartialActionModelState::Trained,
                    PartialActionModelState::Trained
                )
                | (
                    PartialActionModelState::Evaluating(_),
                    PartialActionModelState::Evaluating(_)
                )
        )
    }
}

#[derive(Debug)]
pub struct PartialActionModel {
    model: MlModel<'static>,
    successor_generator_name: SuccessorGeneratorName,
    graph_compiler_name: PartialActionCompilerName,
    wl: WlKernel,
    validate: bool,
    state: PartialActionModelState,
    /// The configuration used to create the model, saved for later use such as
    /// deserialisation
    config: PartialActionModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerialisablePartialActionModel {
    successor_generator_name: SuccessorGeneratorName,
    graph_compiler_name: PartialActionCompilerName,
    wl: WlKernel,
    validate: bool,
    state: PartialActionModelState,
    config: PartialActionModelConfig,
}

impl PartialActionModel {
    pub fn new(py: Python<'static>, config: PartialActionModelConfig) -> Self {
        Self {
            model: MlModel::new(py, config.model),
            successor_generator_name: config.successor_generator,
            graph_compiler_name: config.graph_compiler,
            wl: WlKernel::new(&config.wl),
            validate: config.validate,
            state: PartialActionModelState::New,
            config: config.clone(),
        }
    }

    /// Inspect the weights of the model. This should only be used for debugging
    /// and understanding the model.
    pub fn get_weights(&self) -> Vec<f64> {
        self.model.get_weights(self.config.model).unwrap()
    }

    pub fn inspect_colour(&self, colour: i32) -> Option<&Neighbourhood> {
        self.wl.inspect_colour(colour)
    }

    fn prepare_ranking_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> RankingTrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();
        // TODO: Implement group ids
        let mut group_ids = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = self
                .graph_compiler_name
                .create(task, self.successor_generator_name);

            let mut cur_state = task.initial_state.clone();

            let mut predecessor_graph: Option<CGraph> = None;
            let mut predecessor_group_id: Option<usize> = None;
            for chosen_action in plan.steps() {
                let applicable_actions: Vec<Action> = task
                    .action_schemas()
                    .iter()
                    .flat_map(|schema| -> Vec<Action> {
                        successor_generator.get_applicable_actions(&cur_state, schema)
                    })
                    .collect();

                for partial_depth in 0..=(chosen_action.instantiation.len()) {
                    let partial = PartialAction::from_action(chosen_action, partial_depth);

                    let graph = compiler.compile(&cur_state, &partial);
                    let cur_index = graphs.len();
                    graphs.push(graph.clone());
                    group_ids.push(partial.group_id());

                    // First rank this partial action better than its predecessor
                    if let Some(predecessor_graph) = &predecessor_graph {
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: graphs.len(),
                            relation: RankingRelation::Better,
                        });
                        graphs.push(predecessor_graph.clone());
                        group_ids.push(predecessor_group_id.unwrap());
                    }

                    // Then rank this partial action better than its siblings
                    let siblings: HashSet<PartialAction> =
                        Self::get_siblings(&applicable_actions, &partial, partial_depth);
                    assert!(siblings.contains(&partial));
                    if siblings.len() == 1 {
                        continue;
                    }
                    for sibling in siblings {
                        if sibling == partial {
                            continue;
                        }
                        pairs.push(RankingPair {
                            i: cur_index,
                            j: graphs.len(),
                            relation: RankingRelation::BetterOrEqual,
                        });
                        graphs.push(compiler.compile(&cur_state, &sibling));
                        group_ids.push(sibling.group_id());
                    }

                    predecessor_graph = Some(graph);
                    predecessor_group_id = Some(partial.group_id());
                }

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
            group_ids: Some(group_ids),
        }
    }

    fn prepare_regression_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> RegressionTrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut labels = Vec::new();
        let mut noise = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = self
                .graph_compiler_name
                .create(task, self.successor_generator_name);

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
                        // TODO we hardcode this value for now, but what should
                        // it actually be?
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

        RegressionTrainingData {
            features: graphs,
            labels,
            noise: Some(noise),
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

    fn get_siblings(
        applicable_actions: &[Action],
        chosen_partial: &PartialAction,
        partial_depth: usize,
    ) -> HashSet<PartialAction> {
        // The siblings are all applicable partial actions that have the same
        // prefix as the chosen partial action for depth partial_depth - 1.
        if partial_depth == 0 {
            applicable_actions
                .iter()
                .map(|action| PartialAction::from_action(action, 0))
                .collect()
        } else {
            applicable_actions
                .iter()
                .filter_map(|action| {
                    if action.index != chosen_partial.schema_index() {
                        return None;
                    }

                    let partial = PartialAction::from_action(action, partial_depth - 1);
                    if partial.is_superset_of(chosen_partial) {
                        Some(PartialAction::from_action(action, partial_depth))
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for PartialActionModel {
    fn train(&mut self, train_instances: &[TrainingInstance]) {
        assert_eq!(self.state, PartialActionModelState::New);
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
        let train_graphs = train_data.features();
        let mean_train_graph_size = train_graphs.iter().map(|g| g.node_count()).sum::<usize>()
            as f64
            / train_graphs.len() as f64;
        let val_data = self.prepare_data(val_instances);
        let val_graphs = val_data.features();
        info!("compiled states into graphs");
        info!(
            train_graphs = train_graphs.len(),
            mean_train_graph_size = mean_train_graph_size,
            val_graphs = val_graphs.len()
        );

        let train_histograms = self.wl.compute_histograms(train_graphs);
        let val_histograms = self.wl.compute_histograms(val_graphs);
        info!("computed histograms");

        let train_x = self.wl.compute_x(self.py(), &train_histograms);
        let val_x = self.wl.compute_x(self.py(), &val_histograms);
        info!("computed WL features");
        self.wl.finalise();

        info!(
            train_x_shape = format!("{:?}", train_x.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
        );
        info!("fitting model on training data");

        let train_data = train_data.with_features(train_x);
        let val_data = val_data.with_features(val_x);
        self.model.fit(&train_data);

        if let Some(weights) = self.model.get_weights(self.config.model) {
            const THRESHOLD: f64 = 1e-2;
            let non_zero_weights = weights.iter().filter(|w| w.abs() > THRESHOLD).count();
            info!(
                non_zero_weights = non_zero_weights,
                total_weights = weights.len(),
                sparsity = non_zero_weights as f64 / weights.len() as f64
            );
        }

        let train_score_start = std::time::Instant::now();
        let train_score = self.model.score(&train_data);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        match &self.model {
            MlModel::Regressor(_) => info!(train_mse = train_score),
            MlModel::Ranker(_) => info!(kendall_tau = train_score),
        }

        if self.validate {
            let val_score_start = std::time::Instant::now();
            let val_score = self.model.score(&val_data);
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            match &self.model {
                MlModel::Regressor(_) => info!(val_mse = val_score),
                MlModel::Ranker(_) => info!(kendall_tau = val_score),
            }
        }

        self.state = PartialActionModelState::Trained;
    }

    fn save(&self, path: &Path) {
        assert_eq!(self.state, PartialActionModelState::Trained);

        let pickle_file = NamedTempFile::new().expect("Failed to create temporary file");
        let mut ron_file = NamedTempFile::new().expect("Failed to create temporary file");

        self.model.pickle(pickle_file.path());

        let serialisable = SerialisablePartialActionModel {
            successor_generator_name: self.successor_generator_name,
            graph_compiler_name: self.graph_compiler_name,
            wl: self.wl.clone(),
            validate: self.validate,
            state: PartialActionModelState::Trained,
            config: self.config.clone(),
        };
        let serialised = ron::ser::to_string(&serialisable).expect("Failed to serialise model");

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

impl Evaluate for PartialActionModel {
    type EvaluatedType<'a> = (&'a DBState, &'a PartialAction);

    fn set_evaluating_task(&mut self, task: &Task) {
        match &self.state {
            PartialActionModelState::New => {
                panic!("Model not trained yet, cannot set evaluating task");
            }
            PartialActionModelState::Trained => {
                self.state = PartialActionModelState::Evaluating(
                    self.graph_compiler_name
                        .create(task, self.successor_generator_name),
                );
            }
            PartialActionModelState::Evaluating(_) => {}
        }
    }

    fn evaluate(&mut self, &(state, partial_action): &Self::EvaluatedType<'_>) -> f64 {
        let compiler = match &self.state {
            PartialActionModelState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graph = compiler.compile(state, partial_action);
        let histograms = self.wl.compute_histograms(&[graph]);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y: Vec<f64> = self.model.predict(&x).extract().unwrap();
        y[0]
    }

    fn evaluate_batch(&mut self, targets: &[Self::EvaluatedType<'_>]) -> Vec<f64> {
        let compiler = match &self.state {
            PartialActionModelState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graphs = targets
            .iter()
            .map(|&(state, partial_action)| compiler.compile(state, partial_action))
            .collect::<Vec<_>>();
        let histograms = self.wl.compute_histograms(&graphs);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y: Vec<f64> = self.model.predict(&x).extract().unwrap();
        y
    }

    fn load(py: Python<'static>, path: &Path) -> Self {
        let ron_file = extract_from_zip(path, RON_FILE_NAME);
        let file = std::fs::File::open(ron_file).expect("Failed to open model file");
        let serialisable: SerialisablePartialActionModel =
            ron::de::from_reader(file).expect("Failed to deserialise model");
        assert_eq!(serialisable.state, PartialActionModelState::Trained);

        let pickle_file = extract_from_zip(path, PICKLE_FILE_NAME);
        let model = MlModel::unpickle(serialisable.config.model, py, pickle_file.path());

        Self {
            model,
            successor_generator_name: serialisable.successor_generator_name,
            graph_compiler_name: serialisable.graph_compiler_name,
            wl: serialisable.wl,
            validate: serialisable.validate,
            state: serialisable.state,
            config: serialisable.config,
        }
    }
}
