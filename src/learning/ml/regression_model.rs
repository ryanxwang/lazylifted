//! Wrapper around some sklearn regression models
use crate::learning::ml::py_utils;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time;
use tracing::info;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum RegressorName {
    #[serde(rename = "lr")]
    LinearRegressor,
    #[serde(rename = "gpr")]
    GaussianProcessRegressor,
}

impl RegressorName {
    pub fn to_model_str(self) -> &'static str {
        match self {
            RegressorName::LinearRegressor => "lr",
            RegressorName::GaussianProcessRegressor => "gpr",
        }
    }
}

#[derive(Debug)]
pub struct Regressor<'py> {
    model: Bound<'py, PyAny>,
}

impl<'py> Regressor<'py> {
    pub fn new(py: Python<'py>, name: RegressorName) -> Self {
        let code = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/learning/ml/python/regression_model.py"
        ));
        let py_model = PyModule::from_code_bound(py, code, "regression_model", "regression_model")
            .unwrap()
            .getattr("RegressionModel")
            .unwrap();

        Self {
            model: py_model.call1((name.to_model_str(),)).unwrap(),
        }
    }

    pub fn fit(&self, x: &Bound<'py, PyArray2<f64>>, y: &Bound<'py, PyArray1<f64>>) {
        let start_time = time::Instant::now();
        self.model.getattr("fit").unwrap().call1((x, y)).unwrap();
        info!(fitting_time = start_time.elapsed().as_secs_f64());
    }

    pub fn predict(&self, x: &Bound<'py, PyArray2<f64>>) -> Bound<'py, PyArray1<f64>> {
        let y = self
            .model
            .getattr("predict")
            .unwrap()
            .call1((x,))
            .unwrap()
            .extract()
            .unwrap();
        y
    }

    pub fn get_weights(&self) -> Vec<f64> {
        self.model
            .getattr("get_weights")
            .unwrap()
            .call0()
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
    use assert_approx_eq::assert_approx_eq;
    use numpy::PyArrayMethods;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_imports() {
        Python::with_gil(|py| {
            let _ = Regressor::new(py, RegressorName::GaussianProcessRegressor);
            let _ = Regressor::new(py, RegressorName::LinearRegressor);
        });
    }

    #[test]
    fn test_fit_and_predict_for_gpr() {
        Python::with_gil(|py| {
            let regressor = Regressor::new(py, RegressorName::GaussianProcessRegressor);
            let x = PyArray2::from_vec2_bound(py, &[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
            let y = PyArray1::from_vec_bound(py, vec![1.0, 2.0]);
            regressor.fit(&x, &y);

            let x = PyArray2::from_vec2_bound(py, &[vec![5.0, 6.0], vec![7.0, 8.0]]).unwrap();
            let y = regressor.predict(&x);
            assert_eq!(y.len().unwrap(), 2);
            let y = y.to_vec().unwrap();
            assert_approx_eq!(y[0], 3.0, 1e-5);
            assert_approx_eq!(y[1], 4.0, 1e-5);

            // make sure we can get weights
            let weights = regressor.get_weights();
            assert_approx_eq!(weights[0], 0.0, 1e-5);
            assert_approx_eq!(weights[1], 0.5, 1e-5);
        });
    }

    #[test]
    fn test_fit_and_predict_for_lr() {
        Python::with_gil(|py| {
            let regressor = Regressor::new(py, RegressorName::LinearRegressor);
            let x = PyArray2::from_vec2_bound(py, &[vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
            let y = PyArray1::from_vec_bound(py, vec![1.0, 2.0]);
            regressor.fit(&x, &y);

            let x = PyArray2::from_vec2_bound(py, &[vec![5.0, 6.0], vec![7.0, 8.0]]).unwrap();
            let y = regressor.predict(&x);
            assert_eq!(y.len().unwrap(), 2);
            let y = y.to_vec().unwrap();
            assert_approx_eq!(y[0], 3.0, 1e-5);
            assert_approx_eq!(y[1], 4.0, 1e-5);

            let weights = regressor.get_weights();
            assert_approx_eq!(weights[0], 0.25, 1e-5);
            assert_approx_eq!(weights[1], 0.25, 1e-5);
        });
    }
}
