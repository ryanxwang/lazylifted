//! Wrapper around various Python learning-to-rank models
use crate::learning::{ml::py_utils, models::RankingTrainingData};
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum RankerName {
    #[serde(rename = "ranksvm")]
    RankSVM,
    #[serde(rename = "lambdamart")]
    LambdaMart,
    #[serde(rename = "lp")]
    LP,
}

impl RankerName {
    pub fn to_model_str(self) -> &'static str {
        match self {
            RankerName::RankSVM => "ranksvm",
            RankerName::LambdaMart => "lambdamart",
            RankerName::LP => "lp",
        }
    }
}

#[derive(Debug)]
pub struct Ranker<'py> {
    model: Bound<'py, PyAny>,
}

impl<'py> Ranker<'py> {
    pub fn new(py: Python<'py>, name: RankerName) -> Self {
        let py_model = py_utils::get_ranking_model(py);
        Self {
            model: py_model.call1((name.to_model_str(),)).unwrap(),
        }
    }

    pub fn fit(&self, data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>) {
        let start_time = std::time::Instant::now();
        self.model
            .getattr("fit")
            .unwrap()
            .call1((&data.features, &data.pairs_for_python()))
            .unwrap();
        info!(fitting_time = start_time.elapsed().as_secs_f64());
    }

    pub fn predict(&self, x: &Bound<'py, PyArray2<f64>>) -> Bound<'py, PyArray1<f64>> {
        self.model
            .getattr("predict")
            .unwrap()
            .call1((x,))
            .unwrap()
            .extract()
            .unwrap()
    }

    pub fn kendall_tau(&self, data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>) -> f64 {
        self.model
            .getattr("kendall_tau")
            .unwrap()
            .call1((&data.features, &data.pairs_for_python()))
            .unwrap()
            .extract()
            .unwrap()
    }

    pub fn pickle(&self, pickle_path: &Path) {
        py_utils::pickle(self.py(), &self.model, pickle_path);
    }

    pub fn unpickle(py: Python<'py>, pickle_path: &Path) -> Self {
        let model = py_utils::unpickle(py, pickle_path);
        Self { model }
    }

    pub fn py(&self) -> Python<'py> {
        self.model.py()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::models::{RankingPair, RankingRelation};
    use assert_approx_eq::assert_approx_eq;
    use numpy::PyArrayMethods;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_imports() {
        Python::with_gil(|py: Python<'_>| {
            let _ = Ranker::new(py, RankerName::RankSVM);
        })
    }

    fn test_data(py: Python) -> RankingTrainingData<Bound<PyArray2<f64>>> {
        let x = PyArray2::from_vec2_bound(
            py,
            &[
                vec![1.0, 1.0],
                vec![2.0, 2.0],
                vec![1.2, 1.2],
                vec![1.2, 1.2],
                vec![0.9, 0.9],
                vec![2.2, 2.2],
                vec![1.3, 1.3],
            ],
        )
        .unwrap();

        let pairs = vec![
            RankingPair {
                i: 1,
                j: 0,
                relation: RankingRelation::Better,
            },
            RankingPair {
                i: 1,
                j: 2,
                relation: RankingRelation::Better,
            },
            RankingPair {
                i: 1,
                j: 3,
                relation: RankingRelation::Better,
            },
            RankingPair {
                i: 1,
                j: 4,
                relation: RankingRelation::Better,
            },
            RankingPair {
                i: 5,
                j: 6,
                relation: RankingRelation::Better,
            },
        ];

        RankingTrainingData { features: x, pairs }
    }

    #[test]
    #[serial]
    fn test_fit_and_predict_for_ranksvm() {
        Python::with_gil(|py| {
            let ranker = Ranker::new(py, RankerName::RankSVM);
            let data = test_data(py);
            ranker.fit(&data);

            let x =
                PyArray2::from_vec2_bound(py, &[vec![1.1, 1.1], vec![2.1, 2.1], vec![1.0, 1.0]])
                    .unwrap();
            let y = ranker.predict(&x);
            assert_eq!(y.len().unwrap(), 3);
            let y = y.to_vec().unwrap();
            assert!(y[1] < y[0]);
            assert!(y[1] < y[2]);

            let kendall_tau = ranker.kendall_tau(&data);
            assert_approx_eq!(kendall_tau, 1.0);
        })
    }

    #[test]
    #[serial]
    fn test_fit_and_predict_for_lp() {
        Python::with_gil(|py| {
            let ranker = Ranker::new(py, RankerName::LP);
            let data = test_data(py);
            ranker.fit(&data);

            let x =
                PyArray2::from_vec2_bound(py, &[vec![1.1, 1.1], vec![2.1, 2.1], vec![1.0, 1.0]])
                    .unwrap();
            let y = ranker.predict(&x);
            assert_eq!(y.len().unwrap(), 3);
            let y = y.to_vec().unwrap();
            assert!(y[1] < y[0]);
            assert!(y[1] < y[2]);

            let kendall_tau = ranker.kendall_tau(&data);
            assert_approx_eq!(kendall_tau, 1.0);
        })
    }
}
