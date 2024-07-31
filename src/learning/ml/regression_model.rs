//! Wrapper around some sklearn regression models
use crate::learning::ml::py_utils;
use crate::learning::models::RegressionTrainingData;
use ndarray::{Array1, Array2};
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum RegressorName {
    #[serde(rename = "lr")]
    LinearRegressor,
    #[serde(rename = "gpr")]
    GaussianProcessRegressor { alpha: f64 },
}

impl RegressorName {
    pub fn get_model_str(&self) -> &'static str {
        match self {
            RegressorName::LinearRegressor => "lr",
            RegressorName::GaussianProcessRegressor { alpha: _ } => "gpr",
        }
    }

    pub fn get_kwargs(&self) -> Bound<PyDict> {
        let py = unsafe { Python::assume_gil_acquired() };
        let kwargs = PyDict::new_bound(py);
        match self {
            RegressorName::LinearRegressor => kwargs,
            RegressorName::GaussianProcessRegressor { alpha } => {
                kwargs.set_item("alpha", alpha).unwrap();
                kwargs
            }
        }
    }
}

#[derive(Debug)]
pub struct Regressor<'py> {
    model: Bound<'py, PyAny>,
    // We don't store the bias, so for some models (LR) this could produce
    // slightly different results than the original model. So far this doesn't
    // matter as we are doing GBFS
    weights: Option<Array1<f64>>,
}

impl<'py> Regressor<'py> {
    pub fn new(py: Python<'py>, name: RegressorName) -> Self {
        let py_model = py_utils::get_regression_model(py);
        Self {
            model: py_model
                .call((name.get_model_str(),), Some(&name.get_kwargs()))
                .unwrap(),
            weights: None,
        }
    }

    pub fn fit(&mut self, data: &RegressionTrainingData<Bound<'py, PyArray2<f64>>>) {
        let start_time = time::Instant::now();

        let kwargs = PyDict::new_bound(self.py());
        if let Some(noise) = &data.noise {
            kwargs
                .set_item("noise", PyArray1::from_vec_bound(self.py(), noise.clone()))
                .unwrap();
        }

        self.model
            .getattr("fit")
            .unwrap()
            .call((&data.features, &data.numpy_labels()), Some(&kwargs))
            .unwrap();

        info!(fitting_time = start_time.elapsed().as_secs_f64());

        let weights: Vec<f64> = self
            .model
            .getattr("get_weights")
            .unwrap()
            .call0()
            .unwrap()
            .extract()
            .unwrap();
        self.weights = Some(Array1::from(weights));
    }

    pub fn predict(&self, x: &Array2<f64>) -> Array1<f64> {
        x.dot(&self.weights.as_ref().unwrap().t())
    }

    pub fn score(&self, data: &RegressionTrainingData<Bound<'py, PyArray2<f64>>>) -> f64 {
        self.model
            .getattr("score")
            .unwrap()
            .call1((&data.features, &data.numpy_labels()))
            .unwrap()
            .extract()
            .unwrap()
    }

    pub fn pickle(&self, pickle_path: &Path) {
        py_utils::pickle(self.py(), &self.model, pickle_path);
    }

    pub fn unpickle(py: Python<'py>, pickle_path: &Path) -> Self {
        let model = py_utils::unpickle(py, pickle_path);

        let weights: Vec<f64> = model
            .getattr("get_weights")
            .unwrap()
            .call0()
            .unwrap()
            .extract()
            .unwrap();

        Self {
            model,
            weights: Some(Array1::from(weights)),
        }
    }

    pub fn py(&self) -> Python<'py> {
        self.model.py()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_imports() {
        Python::with_gil(|py| {
            let _ = Regressor::new(py, RegressorName::GaussianProcessRegressor { alpha: 1e-7 });
            let _ = Regressor::new(py, RegressorName::LinearRegressor);
        });
    }

    #[test]
    #[serial]
    fn test_fit_predict_score_for_gpr() {
        Python::with_gil(|py| {
            let mut regressor =
                Regressor::new(py, RegressorName::GaussianProcessRegressor { alpha: 1e-7 });
            let x = PyArray2::from_vec2_bound(py, &[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
            let y = vec![1.0, 2.0];
            let data = RegressionTrainingData {
                features: x,
                labels: y,
                noise: None,
            };
            regressor.fit(&data);

            let score = regressor.score(&data);
            assert_approx_eq!(score, 0.0, 1e-5);

            let x = Array2::from_shape_vec((2, 2), vec![5.0, 6.0, 7.0, 8.0]).unwrap();
            let y = regressor.predict(&x);
            assert_eq!(y.len(), 2);
            assert_approx_eq!(y[0], 3.0, 1e-5);
            assert_approx_eq!(y[1], 4.0, 1e-5);
        });
    }

    #[test]
    #[serial]
    fn test_fit_and_predict_for_lr() {
        Python::with_gil(|py| {
            let mut regressor = Regressor::new(py, RegressorName::LinearRegressor);
            let x = PyArray2::from_vec2_bound(py, &[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
            let y = vec![1.0, 2.0];
            let data = RegressionTrainingData {
                features: x,
                labels: y,
                noise: None,
            };
            regressor.fit(&data);

            let x = Array2::from_shape_vec((2, 2), vec![5.0, 6.0, 7.0, 8.0]).unwrap();
            let y = regressor.predict(&x);
            assert_eq!(y.len(), 2);
            assert_approx_eq!(y[0], 2.75, 1e-5);
            assert_approx_eq!(y[1], 3.75, 1e-5);
        });
    }
}
