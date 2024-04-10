use crate::{
    learning::{
        graphs::{ASLGCompiler, CGraph},
        ml::{Ranker, RankerName},
        models::{Evaluate, Train, TrainingInstance},
        WLKernel,
    },
    search::{successor_generators::SuccessorGeneratorName, ActionSchema, DBState, Task},
};
use numpy::{PyArray1, PyArrayMethods, PyUntypedArrayMethods};
use pyo3::{prelude::*, Python};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    state: WLASLGState,
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

                let mut applicable_schemas_count = 0;
                for schema in &task.action_schemas {
                    if successor_generator
                        .get_applicable_actions(&cur_state, schema)
                        .is_empty()
                    {
                        continue;
                    }
                    applicable_schemas_count += 1;

                    graphs.push(compiler.compile(&cur_state, schema));
                    if schema == chosen_schema {
                        ranks.push(1.0);
                    } else {
                        ranks.push(0.0);
                    }
                }
                groups.push(applicable_schemas_count);

                cur_state = next_state;
            }
        }

        (graphs, ranks, groups)
    }

    fn score(
        &self,
        histograms: &Vec<HashMap<i32, usize>>,
        ranks: &Vec<f64>,
        group: &Vec<usize>,
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

    /// Compute what a baseline model would score for the given training data.
    /// Here baseline means "randomly picking an applicable action schema for
    /// each state"
    fn compute_baseline_score(&self, training_data: &[TrainingInstance]) -> f64 {
        let mut baseline = 0.;
        let mut total = 0.;
        for instance in training_data {
            let task = &instance.task;
            let successor_generator = self.successor_generator_name.create(task);
            let mut cur_state = task.initial_state.clone();
            for action in instance.plan.steps() {
                let num_applicable_schemas = task
                    .action_schemas
                    .iter()
                    .filter(|&schema| {
                        successor_generator
                            .get_applicable_actions(&cur_state, schema)
                            .len()
                            > 0
                    })
                    .count();
                baseline += 1. / num_applicable_schemas as f64;
                total += 1.;
                let action_schema = &task.action_schemas[action.index];
                cur_state =
                    successor_generator.generate_successor(&cur_state, action_schema, action);
            }
        }
        baseline / total
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

        let train_y = PyArray1::from_vec_bound(self.py(), train_ranks.clone());
        let val_y = PyArray1::from_vec_bound(self.py(), val_ranks.clone());
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

        let train_score_start = std::time::Instant::now();
        let train_score = self.score(&train_histograms, &train_ranks, &train_groups);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        let train_baseline = self.compute_baseline_score(train_instances);
        info!(
            train_score = train_score,
            train_baseline = train_baseline,
            train_improvement = train_score - train_baseline
        );

        if self.validate {
            let val_score_start = std::time::Instant::now();
            let val_score = self.score(&val_histograms, &val_ranks, &val_groups);
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            let val_baseline = self.compute_baseline_score(val_instances);
            info!(
                val_score = val_score,
                val_baseline = val_baseline,
                val_improvement = val_score - val_baseline
            );
        }

        self.state = WLASLGState::Trained;
    }

    fn save(&self, path: &std::path::PathBuf) {
        assert_eq!(self.state, WLASLGState::Trained);
        let pickle_path = path.with_extension("pkl");
        self.model.pickle(&pickle_path);

        let ron_path = path.with_extension("ron");
        let serialisable = SerialisableWLASLGModel {
            successor_generator_name: self.successor_generator_name,
            wl: self.wl.clone(),
            validate: self.validate,
            state: self.state.clone(),
        };
        let serialised = ron::ser::to_string(&serialisable).expect("Failed to serialise model");

        let mut file = std::fs::File::create(ron_path).expect("Failed to create model file");
        file.write_all(serialised.as_bytes())
            .expect("Failed to write model data");
        info!("saved model to {}.{{ron/pkl}}", path.display());
    }
}

impl Evaluate for WLASLGModel {
    type EvaluatedType<'a> = (&'a DBState, &'a ActionSchema);

    fn set_evaluating_task(&mut self, task: &Task) {
        match &self.state {
            WLASLGState::New => {
                panic!("Model not trained yet, cannot set evaluating task");
            }
            WLASLGState::Trained => {
                self.state = WLASLGState::Evaluating(ASLGCompiler::new(task));
            }
            WLASLGState::Evaluating(_) => {}
        }
    }

    fn evaluate<'a>(&mut self, &(state, action_schema): &Self::EvaluatedType<'a>) -> f64 {
        let compiler = match &self.state {
            WLASLGState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graph = compiler.compile(state, action_schema);
        let histograms = self.wl.compute_histograms(&[graph]);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y: Vec<f64> = self.model.predict(&x).extract().unwrap();
        y[0]
    }

    fn evaluate_batch<'a>(&mut self, targets: &[Self::EvaluatedType<'a>]) -> Vec<f64> {
        let compiler = match &self.state {
            WLASLGState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graphs = targets
            .iter()
            .map(|&(state, action_schema)| compiler.compile(state, action_schema))
            .collect::<Vec<_>>();
        let histograms = self.wl.compute_histograms(&graphs);
        let x = self.wl.compute_x(self.py(), &histograms);
        let y: Vec<f64> = self.model.predict(&x).extract().unwrap();
        y
    }

    fn load(py: Python<'static>, path: &std::path::PathBuf) -> Self {
        let pickle_path = path.with_extension("pkl");
        let model = Ranker::unpickle(py, &pickle_path);

        let ron_path = path.with_extension("ron");
        let file = std::fs::File::open(ron_path).expect("Failed to open model file");
        let serialisable: SerialisableWLASLGModel =
            ron::de::from_reader(file).expect("Failed to deserialise model");
        assert_eq!(serialisable.state, WLASLGState::Trained);
        Self {
            model,
            successor_generator_name: serialisable.successor_generator_name,
            wl: serialisable.wl,
            validate: serialisable.validate,
            state: serialisable.state,
        }
    }
}
