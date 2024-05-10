use std::path::Path;

use crate::learning::{
    ml::{Ranker, RankerName, Regressor, RegressorName},
    models::TrainingData,
};
use numpy::{PyArray1, PyArray2};
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

    pub fn fit(
        &self,
        training_data: &TrainingData<Bound<'py, PyArray2<f64>>, Bound<'py, PyArray1<f64>>>,
    ) {
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

    pub fn predict(&self, x: &Bound<'py, PyArray2<f64>>) -> Bound<'py, PyArray1<f64>> {
        match self {
            MlModel::Regressor(regressor) => regressor.predict(x),
            MlModel::Ranker(ranker) => ranker.predict(x),
        }
    }

    pub fn score(
        &self,
        data: &TrainingData<Bound<'py, PyArray2<f64>>, Bound<'py, PyArray1<f64>>>,
    ) -> f64 {
        match self {
            MlModel::Regressor(regressor) => match data {
                TrainingData::Regression(data) => regressor.score(data),
                _ => panic!("Wrong data type for regressor model"),
            },
            MlModel::Ranker(ranker) => match data {
                TrainingData::Ranking(data) => ranker.score(data),
                _ => panic!("Wrong data type for ranker model"),
            },
        }
    }

    pub fn get_weights(&self, model_name: MlModelName) -> Vec<f64> {
        match model_name {
            MlModelName::RegressorName(_regressor_name) => {
                let regressor = match self {
                    MlModel::Regressor(regressor) => regressor,
                    _ => panic!("Model does not match provided model name"),
                };
                regressor.get_weights()
            }
            MlModelName::RankerName(_ranker_name) => {
                todo!("Implement get_weights for ranker")
            }
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
