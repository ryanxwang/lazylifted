use crate::{
    learning::{
        data_generators::{DataGenerator, DataGeneratorConfig},
        graphs::{CGraph, ColourDictionary, PartialActionCompilerName},
        ml::MlModel,
        models::{
            model_utils::{extract_from_zip, zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
            preprocessor::Preprocessor,
            wl_model_config::WlModelConfig,
            Evaluate, Train, TrainingInstance,
        },
        wl::WlKernel,
    },
    search::successor_generators::SuccessorGeneratorName,
};
use pyo3::Python;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, time};
use tempfile::NamedTempFile;
use tracing::info;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum WlModelState {
    /// The model has been created but not trained
    New,
    /// Trained byt not ready for evaluation
    Trained,
}

#[derive(Debug)]
pub struct WlModel {
    model: MlModel<'static>,
    wl: WlKernel,
    preprocessor: Preprocessor,
    state: WlModelState,
    /// The configuration used to create the model, saved for later use such as
    /// deserialisation
    config: WlModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerialisableWlModel {
    wl: WlKernel,
    preprocessor: Preprocessor,
    state: WlModelState,
    config: WlModelConfig,
}

impl WlModel {
    pub fn new(py: Python<'static>, config: WlModelConfig) -> Self {
        Self {
            model: MlModel::new(py, config.model),
            wl: WlKernel::new(&config.wl),
            preprocessor: Preprocessor::new(config.preprocessing_option),
            state: WlModelState::New,
            config,
        }
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }

    /// When evaluating, the heuristic needs to know which compiler to use to
    /// input the right graph
    pub fn compiler_name(&self) -> Option<PartialActionCompilerName> {
        match &self.config.data_generator {
            DataGeneratorConfig::PartialSpaceRanking(config) => Some(config.graph_compiler),
            DataGeneratorConfig::PartialSpaceRegression(config) => Some(config.graph_compiler),
            DataGeneratorConfig::PartialSpaceDenseRanking(config) => Some(config.graph_compiler),
            DataGeneratorConfig::StateSpaceIlgRanking(_)
            | DataGeneratorConfig::StateSpaceIlgRegression(_) => None,
        }
    }

    pub fn successor_generator_name(&self) -> SuccessorGeneratorName {
        match &self.config.data_generator {
            // this code is a little silly, but we have to have each one of
            // these lines because each of these "config"s are technically
            // different types
            DataGeneratorConfig::PartialSpaceRanking(config) => config.successor_generator,
            DataGeneratorConfig::PartialSpaceRegression(config) => config.successor_generator,
            DataGeneratorConfig::StateSpaceIlgRanking(config) => config.successor_generator,
            DataGeneratorConfig::StateSpaceIlgRegression(config) => config.successor_generator,
            DataGeneratorConfig::PartialSpaceDenseRanking(config) => config.successor_generator,
        }
    }
}

impl Train for WlModel {
    fn train(&mut self, all_instances: &[TrainingInstance]) {
        assert_eq!(self.state, WlModelState::New);
        if self.config.tune && !self.config.validate {
            panic!("Tuning is only supported when validate is set to true");
        }

        let (train_instances, val_instances) = if self.config.validate || self.config.tune {
            const TRAIN_RATIO: f64 = 0.8;
            info!("splitting train data into train and val sets with train ratio {TRAIN_RATIO:.2}",);
            all_instances.split_at((all_instances.len() as f64 * TRAIN_RATIO) as usize)
        } else {
            info!("training without validation");
            #[allow(trivial_casts)]
            (all_instances, &[] as &[TrainingInstance])
        };

        let mut colour_dictionary = ColourDictionary::new();

        let data_generator = <dyn DataGenerator>::new(&self.config.data_generator);

        let train_data = data_generator.generate(train_instances, &mut colour_dictionary);
        let val_data = data_generator.generate(val_instances, &mut colour_dictionary);
        info!("generated graphs");
        info!(
            train_graph = train_data.features().len(),
            mean_train_graph_size = train_data.mean_graph_size(),
            val_graph = val_data.features().len(),
            mean_val_graph_size = val_data.mean_graph_size(),
        );

        let train_histograms = self
            .preprocessor
            .preprocess(self.wl.compute_histograms(train_data.features()), true);
        let val_histograms = self
            .preprocessor
            .preprocess(self.wl.compute_histograms(val_data.features()), false);
        info!("computed histograms");

        let train_x = self.wl.convert_to_pyarray(self.py(), &train_histograms);
        let val_x = self.wl.convert_to_pyarray(self.py(), &val_histograms);

        let train_data = train_data.with_features(train_x);
        let val_data = val_data.with_features(val_x);

        info!("logging train data");
        train_data.log();
        info!("logging val data");
        val_data.log();

        if self.config.tune {
            info!("tuning model");
            self.model.tune(&train_data, &val_data);

            // for simplicity we just regenerate all data and retrain the model
            // TODO-soon: this needs to happen in train mode of the wl kernal
            let all_data = data_generator.generate(all_instances, &mut colour_dictionary);
            let all_histograms = self
                .preprocessor
                .preprocess(self.wl.compute_histograms(all_data.features()), false);
            let all_x = self.wl.convert_to_pyarray(self.py(), &all_histograms);
            let all_data = all_data.with_features(all_x);

            info!("logging all data");
            all_data.log();

            info!("fitting model");
            self.model.fit(&all_data);
        } else {
            info!("fitting model");
            self.model.fit(&train_data);
        }

        let train_score_start = time::Instant::now();
        let train_score = self.model.score(&train_data);
        info!(train_score_time = train_score_start.elapsed().as_secs_f64());
        match &self.model {
            MlModel::Regressor(_) => info!(train_mse = train_score),
            MlModel::Ranker(_) => info!(kendall_tau = train_score),
        }

        if self.config.validate {
            let val_score_start = time::Instant::now();
            let val_score = self.model.score(&val_data);
            info!(val_score_time = val_score_start.elapsed().as_secs_f64());
            match &self.model {
                MlModel::Regressor(_) => info!(val_mse = val_score),
                MlModel::Ranker(_) => info!(kendall_tau = val_score),
            }
        }

        self.state = WlModelState::Trained;
    }

    fn save(&self, path: &Path) {
        assert_eq!(self.state, WlModelState::Trained);

        let pickle_file = NamedTempFile::new().expect("Failed to create temporary file");
        let mut ron_file = NamedTempFile::new().expect("Failed to create temporary file");

        self.model.pickle(pickle_file.path());

        let serialisable = SerialisableWlModel {
            wl: self.wl.clone(),
            preprocessor: self.preprocessor.clone(),
            state: WlModelState::Trained,
            config: self.config.clone(),
        };
        let serialised = ron::ser::to_string(&serialisable).expect("Failed to serialise model");

        ron_file
            .write_all(serialised.as_bytes())
            .expect("Failed to write serialised model to file");

        zip_files(
            path,
            vec![
                (PICKLE_FILE_NAME, pickle_file.path()),
                (RON_FILE_NAME, ron_file.path()),
            ],
        );
    }
}

impl Evaluate for WlModel {
    type EvaluatedType<'a> = CGraph;

    fn evaluate(&mut self, graph: CGraph, group_id: Option<usize>) -> f64 {
        let histograms = self
            .preprocessor
            .preprocess(self.wl.compute_histograms(&[graph.clone()]), false);
        let x = self.wl.convert_to_ndarray(&histograms);
        self.model.predict_with_ndarray(&x, group_id)[0]
    }

    fn load(py: Python<'static>, path: &Path) -> Self {
        let ron_file = extract_from_zip(path, RON_FILE_NAME);
        let data = std::fs::read_to_string(ron_file).expect("Failed to read model data");
        let serialisable: SerialisableWlModel =
            ron::from_str(&data).expect("Failed to deserialise model data");
        assert_eq!(serialisable.state, WlModelState::Trained);

        let pickle_file = extract_from_zip(path, PICKLE_FILE_NAME);
        let model = MlModel::unpickle(serialisable.config.model, py, pickle_file.path());

        Self {
            model,
            wl: serialisable.wl,
            preprocessor: serialisable.preprocessor,
            state: serialisable.state,
            config: serialisable.config,
        }
    }
}
