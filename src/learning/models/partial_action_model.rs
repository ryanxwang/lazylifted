use crate::{
    learning::{
        graphs::{CGraph, Compiler2, PartialActionCompilerName},
        ml::MlModel,
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            partial_action_model_config::PartialActionModelConfig,
            Evaluate, Train, TrainingInstance,
        },
        wl_kernel::Neighbourhood,
        WlKernel,
    },
    search::{successor_generators::SuccessorGeneratorName, Action, DBState, PartialAction, Task},
};
use numpy::{PyArray1, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::{prelude::*, Python};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::Path,
};
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
    Evaluating(Box<dyn Compiler2<DBState, PartialAction>>),
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
            wl: WlKernel::new(config.iters),
            validate: config.validate,
            state: PartialActionModelState::New,
            config: config.clone(),
        }
    }

    /// Inspect the weights of the model. This should only be used for debugging
    /// and understanding the model.
    pub fn get_weights(&self) -> Vec<f64> {
        self.model.get_weights(self.config.model)
    }

    pub fn inspect_colour(&self, colour: i32) -> Option<&Neighbourhood> {
        self.wl.inspect_colour(colour)
    }

    /// Prepare the data for training from some training instances. The
    /// resulting tuple contains the compiled graphs, the target values (i.e.
    /// ranks), and the groups of the training instances. The groups are used to
    /// indicate the size of each group of data in the other two vectors.
    ///
    /// The groups are generated in various ways to encode the relations that we
    /// would like the model to learn.
    ///
    /// We would like the model to learn to prefer the chosen partial actions
    /// over the other applicable partial actions. This is encoded by creating a
    /// group for each partial action, where the group contains all the
    /// applicable partial actions for the same action schemaa at the same
    /// partial depth.
    ///
    /// We would also like the model to learn to prefer (state, partial) action
    /// pairs that are closer to the goal. For any subsection s_i to s_{i+1} via
    /// a_i and s_{i+1} to s_{i+2} via a_{i+1}, where a_i and a_{i+1} are
    /// partials on the way to the next state, we add a group showing (s_i, a_i)
    /// to be preferred over (s_{i+1}, a_{i+1}).
    fn prepare_ranking_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> (Vec<CGraph>, Vec<f64>, Vec<usize>) {
        let mut graphs = Vec::new();
        let mut ranks = Vec::new();
        let mut groups = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = self
                .graph_compiler_name
                .create(task, self.successor_generator_name);

            let mut cur_state = task.initial_state.clone();
            let mut prev_partials = Vec::new();
            let mut prev_state = None;
            for chosen_action in plan.steps() {
                let applicable_actions: Vec<Action> = task
                    .action_schemas()
                    .iter()
                    .flat_map(|schema| -> Vec<Action> {
                        successor_generator.get_applicable_actions(&cur_state, schema)
                    })
                    .collect();

                // Groups to prefer the chosen partial action over its siblings
                for partial_depth in 0..(chosen_action.instantiation.len() + 1) {
                    let chosen_partial = PartialAction::from_action(chosen_action, partial_depth);
                    let siblings: HashSet<PartialAction> =
                        Self::get_siblings(&applicable_actions, &chosen_partial, partial_depth);
                    assert!(siblings.contains(&chosen_partial));
                    if siblings.len() == 1 {
                        continue;
                    }

                    groups.push(siblings.len());
                    for sibling in siblings {
                        graphs.push(compiler.compile(&cur_state, &sibling));

                        ranks.push(if sibling == chosen_partial { 1.0 } else { 0.0 });
                    }
                }

                // Groups to prefer more specific partials over more general ones
                let partials: Vec<PartialAction> = (0..(chosen_action.instantiation.len() + 1))
                    .map(|depth| PartialAction::from_action(chosen_action, depth))
                    .collect();
                for i in 0..partials.len() - 1 {
                    graphs.push(compiler.compile(&cur_state, &partials[i]));
                    ranks.push(0.0);
                    graphs.push(compiler.compile(&cur_state, &partials[i + 1]));
                    ranks.push(1.0);
                    groups.push(2);
                }

                // Groups to prefer ones closer to the goal
                if let Some(ref prev_state) = prev_state {
                    for previous in &prev_partials {
                        for current in &partials {
                            graphs.push(compiler.compile(prev_state, previous));
                            ranks.push(0.0);
                            graphs.push(compiler.compile(&cur_state, current));
                            ranks.push(1.0);
                            groups.push(2);
                        }
                    }
                }

                prev_partials = partials;
                let next_state = successor_generator.generate_successor(
                    &cur_state,
                    &task.action_schemas()[chosen_action.index],
                    chosen_action,
                );
                (prev_state, cur_state) = (Some(cur_state), next_state);
            }
        }

        (graphs, ranks, groups)
    }

    fn prepare_regression_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> (Vec<CGraph>, Vec<f64>) {
        let mut graphs = Vec::new();
        let mut labels = Vec::new();
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
                }

                cur_state = successor_generator.generate_successor(
                    &cur_state,
                    &task.action_schemas()[chosen_action.index],
                    chosen_action,
                );
            }
        }

        (graphs, labels)
    }

    fn prepare_data(
        &self,
        training_data: &[TrainingInstance],
    ) -> (Vec<CGraph>, Vec<f64>, Option<Vec<usize>>) {
        match self.model {
            MlModel::Regressor(_) => {
                let (graphs, labels) = self.prepare_regression_data(training_data);
                (graphs, labels, None)
            }
            MlModel::Ranker(_) => {
                let (graphs, ranks, groups) = self.prepare_ranking_data(training_data);
                (graphs, ranks, Some(groups))
            }
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

    fn score_ranking(
        &self,
        histograms: &[HashMap<i32, usize>],
        ranks: &[f64],
        group: &[usize],
    ) -> f64 {
        let mut start = 0;
        let mut correct_count = 0;
        for &group_size in group {
            let histogram = &histograms[start..start + group_size];
            let rank = &ranks[start..start + group_size];
            start += group_size;

            let x = self.wl.compute_x(self.py(), histogram);
            let expected = rank
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;
            let predicted_y = self.model.predict(&x);
            let predicted = predicted_y
                .to_vec()
                .unwrap()
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;

            if expected == predicted {
                correct_count += 1;
            }
        }

        correct_count as f64 / group.len() as f64
    }

    fn score_regression(
        &self,
        histograms: &[HashMap<i32, usize>],
        expected_y: &Bound<'static, PyArray1<f64>>,
    ) -> f64 {
        let x = self.wl.compute_x(self.py(), histograms);
        let predicted_y = self.model.predict(&x);
        let mean_squared_error = PyModule::import_bound(self.py(), "sklearn.metrics")
            .unwrap()
            .getattr("mean_squared_error")
            .unwrap();
        let mse = mean_squared_error.call1((expected_y, predicted_y)).unwrap();
        mse.extract().unwrap()
    }

    /// Compute what a baseline model would score for the given training data.
    /// Here baseline means "randomly picking an applicable action schema for
    /// each state"
    fn compute_ranking_baseline(&self, groups: &[usize]) -> f64 {
        let mut baseline = 0.;
        let mut total = 0.;
        for group in groups {
            baseline += 1. / *group as f64;
            total += 1.;
        }
        baseline / total
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for PartialActionModel {
    fn train(&mut self, training_data: &[TrainingInstance]) {
        assert_eq!(self.state, PartialActionModelState::New);
        if self.validate {
            info!("splitting training data into training and validation sets");
        } else {
            info!("training on full dataset");
        }
        let (train_instances, val_instances) = match self.validate {
            true => training_data.split_at((training_data.len() as f64 * 0.8) as usize),
            // Without this trivial cast we get a dumb error message
            #[allow(trivial_casts)]
            false => (training_data, &[] as &[TrainingInstance]),
        };

        let (train_graphs, train_ranks, train_groups) = self.prepare_data(train_instances);
        let mean_train_graph_size = train_graphs.iter().map(|g| g.node_count()).sum::<usize>()
            as f64
            / train_graphs.len() as f64;
        let (val_graphs, val_ranks, val_groups) = self.prepare_data(val_instances);
        info!("compiled states into graphs");
        info!(
            train_graphs = train_graphs.len(),
            mean_train_graph_size = mean_train_graph_size,
            val_graphs = val_graphs.len()
        );

        let train_histograms = self.wl.compute_histograms(&train_graphs);
        let val_histograms = self.wl.compute_histograms(&val_graphs);
        info!("computed histograms");

        let train_x = self.wl.compute_x(self.py(), &train_histograms);
        let val_x = self.wl.compute_x(self.py(), &val_histograms);
        info!("computed WL features");
        self.wl.finalise();

        let train_y = PyArray1::from_vec_bound(self.py(), train_ranks.clone());
        let val_y = PyArray1::from_vec_bound(self.py(), val_ranks.clone());
        info!("converted labels to numpy arrays");
        info!(
            train_x_shape = format!("{:?}", train_x.shape()),
            train_y_shape = format!("{:?}", train_y.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
            val_y_shape = format!("{:?}", val_y.shape()),
            train_groups_count = match &train_groups {
                Some(groups) => groups.len(),
                None => 0,
            },
            val_groups_count = match &val_groups {
                Some(groups) => groups.len(),
                None => 0,
            }
        );
        info!("fitting model on training data");
        self.model.fit(&train_x, &train_y, train_groups.as_deref());

        let weights = self.model.get_weights(self.config.model);
        const THRESHOLD: f64 = 1e-2;
        let non_zero_weights = weights.iter().filter(|w| w.abs() > THRESHOLD).count();
        info!(
            non_zero_weights = non_zero_weights,
            total_weights = weights.len(),
            sparsity = non_zero_weights as f64 / weights.len() as f64
        );

        let train_score_start = std::time::Instant::now();
        let train_score = match &self.model {
            MlModel::Regressor(_) => self.score_regression(&train_histograms, &train_y),
            MlModel::Ranker(_) => {
                let train_groups = train_groups.as_ref().unwrap();
                self.score_ranking(&train_histograms, &train_ranks, train_groups)
            }
        };
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        match &self.model {
            MlModel::Regressor(_) => info!(train_mse = train_score),
            MlModel::Ranker(_) => {
                let train_groups = train_groups.as_ref().unwrap();
                let train_baseline = self.compute_ranking_baseline(train_groups);
                info!(
                    train_score = train_score,
                    train_baseline = train_baseline,
                    train_improvement = train_score - train_baseline
                );
            }
        }

        if self.validate {
            let val_score_start = std::time::Instant::now();
            let val_score = match &self.model {
                MlModel::Regressor(_) => self.score_regression(&val_histograms, &val_y),
                MlModel::Ranker(_) => {
                    let val_groups = val_groups.as_ref().unwrap();
                    self.score_ranking(&val_histograms, &val_ranks, val_groups)
                }
            };
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            match &self.model {
                MlModel::Regressor(_) => info!(val_mse = val_score),
                MlModel::Ranker(_) => {
                    let val_groups = val_groups.as_ref().unwrap();
                    let val_baseline = self.compute_ranking_baseline(val_groups);
                    info!(
                        val_score = val_score,
                        val_baseline = val_baseline,
                        val_improvement = val_score - val_baseline
                    );
                }
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
