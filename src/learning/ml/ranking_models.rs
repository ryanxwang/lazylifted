//! Wrapper around various Python learning-to-rank models
use crate::learning::{ml::py_utils, models::RankingTrainingData, VERBOSE};
use core::fmt::Debug;
use ndarray::{Array1, Array2};
use numpy::PyArray2;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum RankerConfig {
    #[serde(rename = "ranksvm")]
    RankSVM { c_value: f64 },
    #[serde(rename = "lp")]
    LP { c_value: f64 },
}

impl RankerConfig {
    pub fn to_model_str(self) -> &'static str {
        match self {
            RankerConfig::RankSVM { c_value: _ } => "ranksvm",
            RankerConfig::LP { c_value: _ } => "lp",
        }
    }

    pub fn get_c_value(&self) -> f64 {
        match self {
            RankerConfig::RankSVM { c_value } => *c_value,
            RankerConfig::LP { c_value } => *c_value,
        }
    }
}

enum RankerWeights {
    None,
    Vector(Array1<f64>),
    VectorByGroup(HashMap<usize, Array1<f64>>),
}

impl Debug for RankerWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RankerWeights::None => write!(f, "None"),
            // The default debug implementation for Array1 is not very helpful
            // as it only prints the first few elements and the last few
            // elements. We convert it to a Vec to print the entire array
            RankerWeights::Vector(weights) => write!(f, "Vector({:?})", weights.to_vec()),
            RankerWeights::VectorByGroup(weights) => {
                write!(
                    f,
                    "VectorByGroup({:?})",
                    weights
                        .iter()
                        .map(|(group_id, weights)| (group_id, weights.to_vec()))
                        .collect::<Vec<_>>()
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Ranker<'py> {
    model: Bound<'py, PyAny>,
    weights: RankerWeights,
}

impl<'py> Ranker<'py> {
    pub fn new(py: Python<'py>, config: RankerConfig) -> Self {
        let py_model = py_utils::get_ranking_model(py);
        Self {
            model: py_model
                .call1((
                    config.to_model_str(),
                    config.get_c_value(),
                    *VERBOSE.get().unwrap_or(&false),
                ))
                .unwrap(),
            weights: RankerWeights::None,
        }
    }

    pub fn fit(&mut self, data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>) {
        let start_time = std::time::Instant::now();
        self.model
            .getattr("fit")
            .unwrap()
            .call1((
                &data.features,
                &data.pairs_for_python(),
                &data.group_ids_for_python(),
            ))
            .unwrap();
        info!(fitting_time = start_time.elapsed().as_secs_f64());

        let weights = self.model.getattr("get_weights").unwrap().call0().unwrap();
        match weights.extract::<Vec<f64>>() {
            Ok(weights) => self.weights = RankerWeights::Vector(Array1::from(weights)),
            Err(_) => {
                let weights = weights.extract::<HashMap<usize, Vec<f64>>>().unwrap();
                self.weights = RankerWeights::VectorByGroup(
                    weights
                        .into_iter()
                        .map(|(group_id, weights)| (group_id, Array1::from(weights)))
                        .collect(),
                );
            }
        }

        if *VERBOSE.get().unwrap_or(&false) {
            print!("Weights: {:?}", self.weights);
        }
    }

    pub fn tune(
        &mut self,
        training_data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>,
        validation_data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>,
    ) {
        let start_time = std::time::Instant::now();
        self.model
            .getattr("tune")
            .unwrap()
            .call1((
                &training_data.features,
                &training_data.pairs_for_python(),
                &validation_data.features,
                &validation_data.pairs_for_python(),
            ))
            .unwrap();
        info!(tuning_time = start_time.elapsed().as_secs_f64());
    }

    pub fn predict_with_ndarray(&self, x: &Array2<f64>, group_id: Option<usize>) -> Array1<f64> {
        match &self.weights {
            RankerWeights::Vector(weights) => x.dot(&weights.t()),
            RankerWeights::VectorByGroup(weights) => {
                let weights = match group_id {
                    Some(group_id) => weights.get(&group_id).unwrap(),
                    None => panic!("Group ID required for model with group weights"),
                };
                x.dot(&weights.t())
            }
            RankerWeights::None => panic!("Model has not been fit yet"),
        }
    }

    pub fn kendall_tau(&self, data: &RankingTrainingData<Bound<'py, PyArray2<f64>>>) -> f64 {
        self.model
            .getattr("kendall_tau")
            .unwrap()
            .call1((
                &data.features,
                &data.pairs_for_python(),
                &data.group_ids_for_python(),
            ))
            .unwrap()
            .extract()
            .unwrap()
    }

    pub fn pickle(&self, pickle_path: &Path) {
        py_utils::pickle(self.py(), &self.model, pickle_path);
    }

    pub fn unpickle(py: Python<'py>, pickle_path: &Path) -> Self {
        let model = py_utils::unpickle(py, pickle_path);

        let weights = model.getattr("get_weights").unwrap().call0().unwrap();

        match weights.extract::<Vec<f64>>() {
            Ok(weights) => Self {
                model,
                weights: RankerWeights::Vector(Array1::from(weights)),
            },
            Err(_) => {
                let weights = weights.extract::<HashMap<usize, Vec<f64>>>().unwrap();
                Self {
                    model,
                    weights: RankerWeights::VectorByGroup(
                        weights
                            .into_iter()
                            .map(|(group_id, weights)| (group_id, Array1::from(weights)))
                            .collect(),
                    ),
                }
            }
        }
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
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_imports() {
        Python::with_gil(|py: Python<'_>| {
            let _ = Ranker::new(py, RankerConfig::RankSVM { c_value: 1.0 });
        })
    }

    fn test_x(py: Python) -> Bound<PyArray2<f64>> {
        PyArray2::from_vec2_bound(
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
        .unwrap()
    }

    fn test_pairs() -> Vec<RankingPair> {
        vec![
            RankingPair {
                i: 1,
                j: 0,
                relation: RankingRelation::Better,
                importance: 1.0,
            },
            RankingPair {
                i: 1,
                j: 2,
                relation: RankingRelation::Better,
                importance: 1.0,
            },
            RankingPair {
                i: 1,
                j: 3,
                relation: RankingRelation::Better,
                importance: 1.0,
            },
            RankingPair {
                i: 1,
                j: 4,
                relation: RankingRelation::Better,
                importance: 1.0,
            },
            RankingPair {
                i: 5,
                j: 6,
                relation: RankingRelation::Better,
                importance: 1.0,
            },
        ]
    }

    fn test_data_without_groups(py: Python) -> RankingTrainingData<Bound<PyArray2<f64>>> {
        RankingTrainingData {
            features: test_x(py),
            pairs: test_pairs(),
            group_ids: None,
        }
    }

    #[test]
    #[serial]
    fn test_fit_and_predict_for_ranksvm() {
        Python::with_gil(|py| {
            let mut ranker = Ranker::new(py, RankerConfig::RankSVM { c_value: 1.0 });
            let data = test_data_without_groups(py);
            ranker.fit(&data);

            let x = Array2::from_shape_vec((3, 2), vec![1.1, 1.1, 2.1, 2.1, 1.0, 1.0]).unwrap();
            let y = ranker.predict_with_ndarray(&x, None);
            assert_eq!(y.len(), 3);
            assert!(y[1] < y[0]);
            assert!(y[1] < y[2]);
        })
    }

    #[test]
    #[serial]
    fn test_fit_and_predict_for_lp_without_groups() {
        Python::with_gil(|py| {
            let mut ranker = Ranker::new(py, RankerConfig::LP { c_value: 1.0 });
            let data = test_data_without_groups(py);
            ranker.fit(&data);

            let x = Array2::from_shape_vec((3, 2), vec![1.1, 1.1, 2.1, 2.1, 1.0, 1.0]).unwrap();
            let y = ranker.predict_with_ndarray(&x, None);
            assert_eq!(y.len(), 3);
            assert!(y[1] < y[0]);
            assert!(y[1] < y[2]);

            let kendall_tau = ranker.kendall_tau(&data);
            assert_approx_eq!(kendall_tau, 1.0);
        })
    }
}
