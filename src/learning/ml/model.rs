use std::path::Path;

use crate::learning::{
    ml::{Ranker, RankerConfig, Regressor, RegressorConfig},
    models::TrainingData,
};
use ndarray::{Array1, Array2};
use pyo3::{Bound, PyAny, Python};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MlModelConfig {
    Regressor(RegressorConfig),
    Ranker(RankerConfig),
}

#[derive(Debug)]
pub enum MlModel<'py> {
    Regressor(Regressor<'py>),
    Ranker(Ranker<'py>),
}

impl<'py> MlModel<'py> {
    pub fn new(py: Python<'py>, config: MlModelConfig) -> Self {
        match config {
            MlModelConfig::Regressor(config) => MlModel::Regressor(Regressor::new(py, config)),
            MlModelConfig::Ranker(config) => MlModel::Ranker(Ranker::new(py, config)),
        }
    }

    pub fn set_feature_dim(&mut self, dim: usize) {
        match self {
            MlModel::Regressor(regressor) => regressor.set_feature_dim(dim),
            MlModel::Ranker(ranker) => ranker.set_feature_dim(dim),
        }
    }

    pub fn supports_sparse_inputs(&self) -> bool {
        match self {
            MlModel::Regressor(regressor) => regressor.supports_sparse_inputs(),
            MlModel::Ranker(ranker) => ranker.supports_sparse_inputs(),
        }
    }

    pub fn fit(&mut self, training_data: &TrainingData<Bound<'py, PyAny>>) {
        match self {
            MlModel::Regressor(regressor) => match training_data {
                TrainingData::Regression(data) => regressor.fit(data),
                _ => panic!("Wrong data type for regressor model"),
            },
            MlModel::Ranker(ranker) => match training_data {
                TrainingData::Ranking(data) => ranker.fit(data),
                _ => panic!("Wrong data type for ranker model"),
            },
        }
    }

    pub fn tune(
        &mut self,
        training_data: &TrainingData<Bound<'py, PyAny>>,
        validation_data: &TrainingData<Bound<'py, PyAny>>,
    ) {
        match self {
            MlModel::Regressor(_) => {
                panic!("Tuning is not supported for regressor model")
            }
            MlModel::Ranker(ranker) => {
                let training_data = match training_data {
                    TrainingData::Ranking(data) => data,
                    _ => panic!("Wrong data type for ranker model"),
                };
                let validation_data = match validation_data {
                    TrainingData::Ranking(data) => data,
                    _ => panic!("Wrong data type for ranker model"),
                };
                ranker.tune(training_data, validation_data)
            }
        }
    }

    pub fn predict_with_ndarray(&self, x: &Array2<f64>, group_id: Option<usize>) -> Array1<f64> {
        match self {
            MlModel::Regressor(regressor) => regressor.predict(x),
            MlModel::Ranker(ranker) => ranker.predict_with_ndarray(x, group_id),
        }
    }

    pub fn score(&self, data: &TrainingData<Bound<'py, PyAny>>) -> f64 {
        match self {
            MlModel::Regressor(regressor) => match data {
                TrainingData::Regression(data) => regressor.score(data),
                _ => panic!("Wrong data type for regressor model"),
            },
            MlModel::Ranker(ranker) => match data {
                TrainingData::Ranking(data) => ranker.kendall_tau(data),
                _ => panic!("Wrong data type for ranker model"),
            },
        }
    }

    pub fn pickle(&self, path: &Path) {
        match self {
            MlModel::Regressor(regressor) => regressor.pickle(path),
            MlModel::Ranker(ranker) => ranker.pickle(path),
        }
    }

    pub fn unpickle(model_name: MlModelConfig, py: Python<'py>, path: &Path) -> Self {
        match model_name {
            MlModelConfig::Regressor(_config) => MlModel::Regressor(Regressor::unpickle(py, path)),
            MlModelConfig::Ranker(_config) => MlModel::Ranker(Ranker::unpickle(py, path)),
        }
    }

    pub fn py(&self) -> Python<'py> {
        match self {
            MlModel::Regressor(regressor) => regressor.py(),
            MlModel::Ranker(ranker) => ranker.py(),
        }
    }
}
