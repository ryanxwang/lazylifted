use super::{wl_model_config::WlModelConfig, Train, TrainingInstance};
use crate::learning::{
    data_generators::DataGenerator,
    ml::MlModel,
    models::model_utils::{zip_files, PICKLE_FILE_NAME, RON_FILE_NAME},
    wl::WlKernel,
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
    state: WlModelState,
    /// The configuration used to create the model, saved for later use such as
    /// deserialisation
    config: WlModelConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerialisableWlModel {
    wl: WlKernel,
    state: WlModelState,
    config: WlModelConfig,
}

impl WlModel {
    pub fn new(py: Python<'static>, config: WlModelConfig) -> Self {
        Self {
            model: MlModel::new(py, config.model),
            wl: WlKernel::new(&config.wl),
            state: WlModelState::New,
            config,
        }
    }

    fn py(&self) -> Python<'static> {
        self.model.py()
    }
}

impl Train for WlModel {
    fn train(&mut self, train_instances: &[TrainingInstance]) {
        assert_eq!(self.state, WlModelState::New);
        let (train_instances, val_instances) = if self.config.validate {
            const TRAIN_RATIO: f64 = 0.8;
            info!("splitting train data into train and val sets with train ratio {TRAIN_RATIO:.2}",);
            train_instances.split_at((train_instances.len() as f64 * TRAIN_RATIO) as usize)
        } else {
            info!("training without validation");
            #[allow(trivial_casts)]
            (train_instances, &[] as &[TrainingInstance])
        };

        let data_generator = <dyn DataGenerator>::new(&self.config.data_generator);

        let train_data = data_generator.generate(train_instances);
        let val_data = data_generator.generate(val_instances);
        info!("generated graphs");
        info!(
            train_graph = train_data.features().len(),
            mean_train_graph_size = train_data.mean_graph_size(),
            val_graph = val_data.features().len(),
            mean_val_graph_size = val_data.mean_graph_size(),
        );

        let train_histograms = self.wl.compute_histograms(train_data.features());
        let val_histograms = self.wl.compute_histograms(val_data.features());
        info!("computed histograms");

        let train_x = self.wl.convert_to_pyarray(self.py(), &train_histograms);
        let val_x = self.wl.convert_to_pyarray(self.py(), &val_histograms);

        let train_data = train_data.with_features(train_x);
        let val_data = val_data.with_features(val_x);

        info!("logging train data");
        train_data.log();
        info!("logging val data");
        val_data.log();

        info!("fitting model");
        self.model.fit(&train_data);

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
