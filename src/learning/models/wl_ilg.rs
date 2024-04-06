use crate::{
    learning::{
        graphs::{CGraph, ILGCompiler},
        ml::{Regressor, RegressorName},
        models::{Train, TrainingInstance},
        WLKernel,
    },
    search::successor_generators::{SuccessorGenerator, SuccessorGeneratorName},
};
use numpy::{PyArray1, PyArray2, PyUntypedArrayMethods};
use pyo3::{
    types::{PyAnyMethods, PyModule},
    Bound, Python,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time};
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WLILGConfig {
    pub model: RegressorName,
    #[serde(alias = "successor-generator")]
    pub successor_generator: SuccessorGeneratorName,
    pub iters: usize,
    pub validate: bool,
}

pub struct WLILGModel<'py> {
    pub model: Regressor<'py>,
    /// The successor generator to use for generating successor states when
    /// training. It might appear weird we store the name of the successor
    /// generator instead of the generator itself, but this is because 1)
    /// it is only used in training, and 2) each task requires its own successor
    /// generator, so we can't store a single instance of the generator.
    pub successor_generator_name: SuccessorGeneratorName,
    pub wl: WLKernel,
    pub validate: bool,
}

impl<'py> WLILGModel<'py> {
    pub fn new(py: Python<'py>, config: WLILGConfig) -> Self {
        Self {
            model: Regressor::new(py, config.model),
            wl: WLKernel::new(config.iters),
            successor_generator_name: config.successor_generator,
            validate: config.validate,
        }
    }

    fn prepare_data(&self, training_data: &[TrainingInstance]) -> (Vec<CGraph>, Vec<f64>) {
        let mut graphs = Vec::new();
        let mut dist_to_goal = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let compiler = ILGCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            for (i, action) in plan.steps().iter().enumerate() {
                let action_schema = &task.action_schemas[action.index];
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
        py: Python<'py>,
        x: Bound<'py, PyArray2<f64>>,
        expected_y: Bound<'py, PyArray1<f64>>,
    ) -> f64 {
        let predicted_y = self.model.predict(&x);
        let mean_squared_error = PyModule::import_bound(py, "sklearn.metrics")
            .unwrap()
            .getattr("mean_squared_error")
            .unwrap();
        let mse = mean_squared_error.call1((expected_y, predicted_y)).unwrap();
        mse.extract().unwrap()
    }
}

impl<'py> Train<'py> for WLILGModel<'py> {
    fn train(&mut self, py: Python<'py>, training_data: &[TrainingInstance]) {
        if self.validate {
            info!(target : "progress", "splitting training data into training and validation sets");
        } else {
            info!(target : "progress", "training on full dataset");
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
        info!(target : "progress", "compiled states into graphs");
        info!(target : "stats", train_graphs = train_graphs.len(), mean_train_graph_size = mean_train_graph_size, val_graphs = val_graphs.len());

        let train_histograms = self.wl.compute_histograms(&train_graphs);
        let val_histograms = self.wl.compute_histograms(&val_graphs);
        info!(target : "progress", "computed WL histograms");
        self.wl.log();

        let train_x = self.wl.compute_x(py, &train_histograms);
        let val_x = self.wl.compute_x(py, &val_histograms);
        info!(target : "progress", "computed WL features");

        let train_y = PyArray1::from_vec_bound(py, train_labels);
        let val_y = PyArray1::from_vec_bound(py, val_labels);
        info!(target : "progress", "converted labels to numpy arrays");
        info!(
            target : "stats",
            train_x_shape = format!("{:?}", train_x.shape()),
            train_y_shape = format!("{:?}", train_y.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
            val_y_shape = format!("{:?}", val_y.shape())
        );

        info!(target : "progress", "fitting model on training data");
        self.model.fit(&train_x, &train_y);

        let train_score_start = time::Instant::now();
        let train_score = self.score(py, train_x, train_y);
        info!(target : "timing", train_score_time = train_score_start.elapsed().as_secs_f64());
        info!(target : "stats", train_score = train_score);
        if self.validate {
            let val_score_start = time::Instant::now();
            let val_score = self.score(py, val_x, val_y);
            info!(target : "timing", val_score_time = val_score_start.elapsed().as_secs_f64());
            info!(target : "stats", val_score = val_score);
        }
    }

    fn save(&self, path: &PathBuf) {
        todo!()
    }
}
