use crate::{
    learning::{
        graphs::{CGraph, IlgCompiler},
        ml::{Regressor, RegressorName},
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            Evaluate, Train, TrainingInstance,
        },
        WlKernel,
    },
    search::{successor_generators::SuccessorGeneratorName, DBState, Task},
};
use numpy::{PyArray1, PyArray2, PyUntypedArrayMethods};
use pyo3::{
    types::{PyAnyMethods, PyModule},
    Bound, Python,
};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, time};
use tempfile::NamedTempFile;
use tracing::info;

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

/// Configuration for the WL-ILG model. This is the format used by the trainer
/// to create the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateSpaceModelConfig {
    pub model: RegressorName,
    pub successor_generator: SuccessorGeneratorName,
    pub iters: usize,
    pub validate: bool,
}

#[derive(Debug)]
pub struct StateSpaceModel {
    model: Regressor<'static>,
    /// The successor generator to use for generating successor states when
    /// training. It might appear weird we store the name of the successor
    /// generator instead of the generator itself, but this is because 1)
    /// it is only used in training, and 2) each task requires its own successor
    /// generator, so we can't store a single instance of the generator.
    successor_generator_name: SuccessorGeneratorName,
    wl: WlKernel,
    validate: bool,
    state: ModelState,
}

/// Dummy struct to allow serialising/deserialising the model to disk.
#[derive(Debug, Serialize, Deserialize)]
struct SerialisableStateSpaceModel {
    successor_generator_name: SuccessorGeneratorName,
    wl: WlKernel,
    validate: bool,
    state: ModelState,
}

impl StateSpaceModel {
    pub fn new(py: Python<'static>, config: StateSpaceModelConfig) -> Self {
        Self {
            model: Regressor::new(py, config.model),
            wl: WlKernel::new(config.iters),
            successor_generator_name: config.successor_generator,
            validate: config.validate,
            state: ModelState::New,
        }
    }

    fn prepare_data(&self, training_data: &[TrainingInstance]) -> (Vec<CGraph>, Vec<f64>) {
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
            // graphs.push(compiler.compile(&cur_state));
            // dist_to_goal.push(0.0);
        }

        assert_eq!(graphs.len(), dist_to_goal.len());
        (graphs, dist_to_goal)
    }

    fn score(
        &self,
        x: Bound<'static, PyArray2<f64>>,
        expected_y: Bound<'static, PyArray1<f64>>,
    ) -> f64 {
        let predicted_y = self.model.predict(&x);
        let mean_squared_error = PyModule::import_bound(self.py(), "sklearn.metrics")
            .unwrap()
            .getattr("mean_squared_error")
            .unwrap();
        let mse = mean_squared_error.call1((expected_y, predicted_y)).unwrap();
        mse.extract().unwrap()
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for StateSpaceModel {
    fn train(&mut self, training_data: &[TrainingInstance]) {
        let py = self.py();
        assert_eq!(self.state, ModelState::New);
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

        let (train_graphs, train_labels) = self.prepare_data(train_instances);
        let mean_train_graph_size = train_graphs.iter().map(|g| g.node_count()).sum::<usize>()
            as f64
            / train_graphs.len() as f64;
        let (val_graphs, val_labels) = self.prepare_data(val_instances);
        info!("compiled states into graphs");
        info!(
            train_graphs = train_graphs.len(),
            mean_train_graph_size = mean_train_graph_size,
            val_graphs = val_graphs.len()
        );

        let train_histograms = self.wl.compute_histograms(&train_graphs);
        let val_histograms = self.wl.compute_histograms(&val_graphs);
        info!("computed WL histograms");

        let train_x = self.wl.compute_x(py, &train_histograms);
        let val_x = self.wl.compute_x(py, &val_histograms);
        info!("computed WL features");
        self.wl.finalise();

        let train_y = PyArray1::from_vec_bound(py, train_labels);
        let val_y = PyArray1::from_vec_bound(py, val_labels);
        info!("converted labels to numpy arrays");
        info!(
            train_x_shape = format!("{:?}", train_x.shape()),
            train_y_shape = format!("{:?}", train_y.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
            val_y_shape = format!("{:?}", val_y.shape())
        );

        info!("fitting model on training data");
        self.model.fit(&train_x, &train_y);

        let train_score_start = time::Instant::now();
        let train_score = self.score(train_x, train_y);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        info!(train_score = train_score);
        if self.validate {
            let val_score_start = time::Instant::now();
            let val_score = self.score(val_x, val_y);
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
        let pickle_file = extract_from_zip(path, PICKLE_FILE_NAME);
        let model = Regressor::unpickle(py, pickle_file.path());

        let ron_file = extract_from_zip(path, RON_FILE_NAME);
        let data = std::fs::read_to_string(ron_file).expect("Failed to read model data");
        let serialisable: SerialisableStateSpaceModel =
            ron::from_str(&data).expect("Failed to deserialise model data");
        assert_eq!(serialisable.state, ModelState::Trained);
        Self {
            model,
            successor_generator_name: serialisable.successor_generator_name,
            wl: serialisable.wl,
            validate: serialisable.validate,
            state: serialisable.state,
        }
    }
}
