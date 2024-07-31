use std::path::Path;

use crate::learning::{
    ml::{Ranker, RankerName, Regressor, RegressorName},
    models::TrainingData,
};
use ndarray::{Array1, Array2};
use numpy::PyArray2;
use pyo3::{Bound, Python};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum MlModelName {
    RegressorName(RegressorName),
    RankerName(RankerName),
}

#[derive(Debug)]
pub enum MlModel<'py> {
    Regressor(Regressor<'py>),
    Ranker(Ranker<'py>),
}

impl<'py> MlModel<'py> {
    pub fn new(py: Python<'py>, name: MlModelName) -> Self {
        match name {
            MlModelName::RegressorName(name) => MlModel::Regressor(Regressor::new(py, name)),
            MlModelName::RankerName(name) => MlModel::Ranker(Ranker::new(py, name)),
        }
    }

    pub fn fit(&mut self, training_data: &TrainingData<Bound<'py, PyArray2<f64>>>) {
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

    pub fn predict_with_ndarray(&self, x: &Array2<f64>, group_id: Option<usize>) -> Array1<f64> {
        match self {
            MlModel::Regressor(regressor) => regressor.predict(x),
            MlModel::Ranker(ranker) => ranker.predict_with_ndarray(x, group_id),
        }
    }

    pub fn score(&self, data: &TrainingData<Bound<'py, PyArray2<f64>>>) -> f64 {
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

    pub fn unpickle(model_name: MlModelName, py: Python<'py>, path: &Path) -> Self {
        match model_name {
            MlModelName::RegressorName(_name) => MlModel::Regressor(Regressor::unpickle(py, path)),
            MlModelName::RankerName(_name) => MlModel::Ranker(Ranker::unpickle(py, path)),
        }
    }

    pub fn py(&self) -> Python<'py> {
        match self {
            MlModel::Regressor(regressor) => regressor.py(),
            MlModel::Ranker(ranker) => ranker.py(),
        }
    }
}
