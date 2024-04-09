use numpy::{PyArray1, PyUntypedArrayMethods};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    learning::{
        graphs::{ASLGCompiler, CGraph},
        ml::{Ranker, RankerName},
        models::{Train, TrainingInstance},
        WLKernel,
    },
    search::successor_generators::SuccessorGeneratorName,
};

#[derive(Debug, Clone)]
enum WLASLGState {
    // The model has been created but not trained
    New,
    // Trained but not ready for evaluating
    Trained,
    // Ready for evaluating
    Evaluating(ASLGCompiler),
}

impl PartialEq for WLASLGState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (WLASLGState::New, WLASLGState::New) => true,
            (WLASLGState::Trained, WLASLGState::Trained) => true,
            (WLASLGState::Evaluating(_), WLASLGState::Evaluating(_)) => true,
            _ => false,
        }
    }
}

/// Configuration for the WL-ASLG model. This is the format used by the trainer
/// to create the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WLASLGConfig {
    pub model: RankerName,
    #[serde(alias = "successor-generator")]
    pub successor_generator: SuccessorGeneratorName,
    pub iters: usize,
    pub validate: bool,
}

#[derive(Debug)]
pub struct WLASLGModel {
    model: Ranker<'static>,
    /// See also [`crate::learning::models::wl_ilg::WLILGModel::successor_generator_name`].
    successor_generator_name: SuccessorGeneratorName,
    wl: WLKernel,
    validate: bool,
    state: WLASLGState,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerialisableWLASLGModel {
    successor_generator_name: SuccessorGeneratorName,
    wl: WLKernel,
    validate: bool,
}

impl WLASLGModel {
    pub fn new(py: Python<'static>, config: WLASLGConfig) -> Self {
        Self {
            model: Ranker::new(py, config.model),
            wl: WLKernel::new(config.iters),
            successor_generator_name: config.successor_generator,
            validate: config.validate,
            state: WLASLGState::New,
        }
    }

    /// Prepare the data for training from some training instances. The resulting
    /// tuple contains the compiled graphs, the target values (i.e. ranks), and
    /// the groups of the training instances. The groups are used to indicate
    /// the size of each group of data in the other two vectors.
    fn prepare_data(
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
            let compiler = ASLGCompiler::new(task);

            let mut cur_state = task.initial_state.clone();
            for action in plan.steps() {
                let chosen_schema = &task.action_schemas[action.index];
                let next_state =
                    successor_generator.generate_successor(&cur_state, chosen_schema, action);

                for schema in &task.action_schemas {
                    graphs.push(compiler.compile(&cur_state, schema));
                    if schema == chosen_schema {
                        ranks.push(1.0);
                    } else {
                        ranks.push(0.0);
                    }
                }
                groups.push(task.action_schemas.len());

                cur_state = next_state;
            }
        }

        (graphs, ranks, groups)
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for WLASLGModel {
    fn train(&mut self, training_data: &[TrainingInstance]) {
        assert_eq!(self.state, WLASLGState::New);
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
        self.wl.log();

        let train_x = self.wl.compute_x(self.py(), &train_histograms);
        let val_x = self.wl.compute_x(self.py(), &val_histograms);
        info!("computed WL features");

        let train_y = PyArray1::from_vec_bound(self.py(), train_ranks);
        let val_y = PyArray1::from_vec_bound(self.py(), val_ranks);
        info!("converted labels to numpy arrays");
        info!(
            train_x_shape = format!("{:?}", train_x.shape()),
            train_y_shape = format!("{:?}", train_y.shape()),
            val_x_shape = format!("{:?}", val_x.shape()),
            val_y_shape = format!("{:?}", val_y.shape()),
            train_groups_count = train_groups.len(),
            val_groups_count = val_groups.len()
        );
        info!("fitted model on training data");
        self.model.fit(&train_x, &train_y, &train_groups);

        todo!("train")
    }

    fn save(&self, _path: &std::path::PathBuf) {
        todo!("Implement saving")
    }
}
