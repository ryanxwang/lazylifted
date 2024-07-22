use std::path::Path;

use crate::{
    learning::{
        graphs::{CGraph, IlgCompiler},
        ml::{Ranker, RankerName},
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            schema_decomposed_model_config::SchemaDecomposedModelConfig,
            Evaluate, RankingPair, RankingRelation, RankingTrainingData, Train, TrainingInstance,
        },
        wl::WlKernel,
    },
    search::{
        states::{SchemaDecomposedState, SchemaOrInstantiation},
        Action, DBState, Task,
    },
};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::io::Write;
use tempfile::NamedTempFile;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
enum SchemaDecomposedModelState {
    // The model has been created but not trained
    New,
    // Trained but not ready for evaluating
    Trained,
    // Ready for evaluating
    #[serde(skip)]
    Evaluating(IlgCompiler),
}

impl PartialEq for SchemaDecomposedModelState {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::New, Self::New)
                | (Self::Trained, Self::Trained)
                | (Self::Evaluating(_), Self::Evaluating(_))
        )
    }
}

#[derive(Debug)]
pub struct SchemaDecomposedModel {
    model: Ranker<'static>,
    wl: WlKernel,
    state: SchemaDecomposedModelState,
    config: SchemaDecomposedModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerialisableSchemaDecomposedModel {
    wl: WlKernel,
    state: SchemaDecomposedModelState,
    config: SchemaDecomposedModelConfig,
}

impl SchemaDecomposedModel {
    pub fn new(py: Python<'static>, config: SchemaDecomposedModelConfig) -> Self {
        let model_name = RankerName::LP;

        Self {
            model: Ranker::new(py, model_name),
            wl: WlKernel::new(&config.wl),
            state: SchemaDecomposedModelState::New,
            config,
        }
    }

    fn prepare_data(&self, training_data: &[TrainingInstance]) -> RankingTrainingData<Vec<CGraph>> {
        let mut graphs = Vec::new();
        let mut pairs = Vec::new();
        let mut group_ids = Vec::new();
        for instance in training_data {
            let plan = &instance.plan;
            let task = &instance.task;
            let successor_generator = self.config.successor_generator.create(task);
            let compiler = IlgCompiler::new(task);

            let mut cur_state = SchemaDecomposedState::without_schema(task.initial_state.clone());
            let mut cur_index = graphs.len();
            graphs.push(compiler.compile(cur_state.state()));
            group_ids.push(cur_state.group_id());

            let transitions = plan
                .steps()
                .iter()
                .flat_map(SchemaOrInstantiation::from_action);
            for transition in transitions {
                let (successor, siblings) = match transition {
                    SchemaOrInstantiation::Schema(schema_index) => {
                        let successor = SchemaDecomposedState::with_schema(
                            cur_state.state().clone(),
                            schema_index,
                        );

                        let mut siblings = Vec::new();
                        for (i, schema) in task.action_schemas().iter().enumerate() {
                            if i == schema_index {
                                continue;
                            }
                            if successor_generator
                                .get_applicable_actions(cur_state.state(), schema)
                                .is_empty()
                            {
                                continue;
                            }
                            siblings.push(SchemaDecomposedState::with_schema(
                                cur_state.state().clone(),
                                i,
                            ));
                        }

                        (successor, siblings)
                    }
                    SchemaOrInstantiation::Instantiation(action) => {
                        assert_eq!(cur_state.schema(), Some(action.index));
                        let action_schema = &task.action_schemas()[cur_state.schema().unwrap()];
                        // let applicable_actions = successor_generator
                        //     .get_applicable_actions(cur_state.state(), action_schema);
                        let applicable_actions: Vec<Action> = task
                            .action_schemas()
                            .iter()
                            .flat_map(|schema| {
                                successor_generator
                                    .get_applicable_actions(cur_state.state(), schema)
                            })
                            .collect();

                        let successor = SchemaDecomposedState::without_schema(
                            successor_generator.generate_successor(
                                cur_state.state(),
                                action_schema,
                                &action,
                            ),
                        );

                        let mut siblings = Vec::new();
                        for applicable_action in applicable_actions {
                            if action == applicable_action {
                                continue;
                            }

                            siblings.push(SchemaDecomposedState::without_schema(
                                successor_generator.generate_successor(
                                    cur_state.state(),
                                    &task.action_schemas()[applicable_action.index],
                                    &applicable_action,
                                ),
                            ));
                        }

                        (successor, siblings)
                    }
                };

                // Successor is better than current state
                let successor_index = graphs.len();
                graphs.push(compiler.compile(successor.state()));
                group_ids.push(successor.group_id());

                pairs.push(RankingPair {
                    i: successor_index,
                    j: cur_index,
                    relation: RankingRelation::Better,
                });

                // Successor is better than or equal to its siblings
                for sibling in siblings {
                    pairs.push(RankingPair {
                        i: successor_index,
                        j: graphs.len(),
                        relation: RankingRelation::BetterOrEqual,
                    });
                    graphs.push(compiler.compile(sibling.state()));
                    group_ids.push(sibling.group_id());
                }

                cur_state = successor;
                cur_index = successor_index;
            }
        }

        RankingTrainingData {
            features: graphs,
            pairs,
            group_ids: Some(group_ids),
        }
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for SchemaDecomposedModel {
    fn train(&mut self, train_instances: &[TrainingInstance]) {
        assert_eq!(self.state, SchemaDecomposedModelState::New);
        if self.config.validate {
            info!("splitting training data into training and validation sets");
        } else {
            info!("training on full dataset");
        }
        let (train_instances, val_instances) = match self.config.validate {
            true => train_instances.split_at((train_instances.len() as f64 * 0.8) as usize),
            // Without this trivial cast we get a dumb error message
            #[allow(trivial_casts)]
            false => (train_instances, &[] as &[TrainingInstance]),
        };

        let train_data = self.prepare_data(train_instances);
        let train_graphs = &train_data.features;
        let mean_train_graph_size = train_graphs.iter().map(|g| g.node_count()).sum::<usize>()
            as f64
            / train_graphs.len() as f64;
        let val_data = self.prepare_data(val_instances);
        let val_graphs = &val_data.features;
        info!("compiled states into graphs");
        info!(
            train_graphs = train_graphs.len(),
            mean_train_graph_size = mean_train_graph_size,
            val_graphs = val_graphs.len()
        );

        let train_histograms = self.wl.compute_histograms(train_graphs);
        let val_histograms = self.wl.compute_histograms(val_graphs);
        info!("computed histograms");

        let train_x = self.wl.convert_to_pyarray(self.py(), &train_histograms);
        let val_x = self.wl.convert_to_pyarray(self.py(), &val_histograms);
        info!("computed WL features");
        self.wl.finalise();

        let train_data = train_data.with_features(train_x);
        let val_data = val_data.with_features(val_x);

        info!("logging training data");
        train_data.log();
        info!("logging validation data");
        val_data.log();

        info!("fitting model on training data");
        self.model.fit(&train_data);

        let train_score_start = std::time::Instant::now();
        let train_score = self.model.kendall_tau(&train_data);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        info!(kendall_tau = train_score);

        if self.config.validate {
            let val_score_start = std::time::Instant::now();
            let val_score = self.model.kendall_tau(&val_data);
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            info!(kendall_tau = val_score)
        }

        self.state = SchemaDecomposedModelState::Trained;
    }

    fn save(&self, path: &Path) {
        assert_eq!(self.state, SchemaDecomposedModelState::Trained);

        let pickle_file = NamedTempFile::new().expect("Failed to create temporary file");
        let mut ron_file = NamedTempFile::new().expect("Failed to create temporary file");

        self.model.pickle(pickle_file.path());

        let serialisable = SerialisableSchemaDecomposedModel {
            wl: self.wl.clone(),
            state: SchemaDecomposedModelState::Trained,
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

impl Evaluate for SchemaDecomposedModel {
    type EvaluatedType<'a> = SchemaDecomposedState<DBState>;

    fn set_evaluating_task(&mut self, task: &Task) {
        match &self.state {
            SchemaDecomposedModelState::New => {
                panic!("Model not trained yet, cannot set evaluating task");
            }
            SchemaDecomposedModelState::Trained => {
                self.state = SchemaDecomposedModelState::Evaluating(IlgCompiler::new(task));
            }
            SchemaDecomposedModelState::Evaluating(_) => {}
        }
    }

    fn evaluate(&mut self, state: &Self::EvaluatedType<'_>) -> f64 {
        let compiler = match &self.state {
            SchemaDecomposedModelState::Evaluating(compiler) => compiler,
            _ => panic!("Model not ready for evaluation"),
        };
        let graph = compiler.compile(state.state());
        let histograms = self.wl.compute_histograms(&[graph]);
        let x = self.wl.convert_to_ndarray(&histograms);

        let y = self.model.predict_with_ndarray(&x, Some(state.group_id()));
        y[0]
    }

    fn load(py: Python<'static>, path: &Path) -> Self {
        let ron_file = extract_from_zip(path, RON_FILE_NAME);
        let file = std::fs::File::open(ron_file).expect("Failed to open model file");
        let serialisable: SerialisableSchemaDecomposedModel =
            ron::de::from_reader(file).expect("Failed to deserialise model");
        assert_eq!(serialisable.state, SchemaDecomposedModelState::Trained);

        let pickle_file = extract_from_zip(path, PICKLE_FILE_NAME);
        let model = Ranker::unpickle(py, pickle_file.path());

        Self {
            model,
            wl: serialisable.wl,
            state: serialisable.state,
            config: serialisable.config,
        }
    }
}
