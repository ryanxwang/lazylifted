use std::path::Path;

use crate::learning::ml::{Ranker, RankerName, Regressor, RegressorName};
use numpy::{PyArray1, PyArray2};
use pyo3::{Bound, Python};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        x: &Bound<'py, PyArray2<f64>>,
        y: &Bound<'py, PyArray1<f64>>,
        group: &Option<Vec<usize>>,
    ) {
        match self {
            MlModel::Regressor(regressor) => regressor.fit(x, y),
            MlModel::Ranker(ranker) => {
                let group = group
                    .as_ref()
                    .expect("Group is required for ranking models");
                ranker.fit(x, y, group)
            }
        }
    }

    pub fn predict(&self, x: &Bound<'py, PyArray2<f64>>) -> Bound<'py, PyArray1<f64>> {
        match self {
            MlModel::Regressor(regressor) => regressor.predict(x),
            MlModel::Ranker(ranker) => ranker.predict(x),
        }
    }

    pub fn pickle(&self, path: &Path) {
        match self {
            MlModel::Regressor(regressor) => regressor.pickle(path),
            MlModel::Ranker(ranker) => ranker.pickle(path),
        }
    }

    pub fn unpickle(model_name: MlModelName, py: Python<'py>, path: &Path) -> Self {
        // TODO the constructors should take in [`_name`]
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
