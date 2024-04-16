//! Wrapper around various Python learning-to-rank models
use crate::learning::ml::py_utils;
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum RankerName {
    #[serde(alias = "ranksvm")]
    RankSVM,
    #[serde(alias = "lambdamart")]
    LambdaMart,
}

#[derive(Debug)]
pub struct Ranker<'py> {
    name: RankerName,
    model: Bound<'py, PyAny>,
}

impl<'py> Ranker<'py> {
    pub fn new(py: Python<'py>, name: RankerName) -> Self {
        Self {
            name: name.clone(),
            model: Self::construct_ranker(py, name),
        }
    }

    pub fn fit(
        &self,
        x: &Bound<'py, PyArray2<f64>>,
        y: &Bound<'py, PyArray1<f64>>,
        group: &Vec<usize>,
    ) {
        let start_time = std::time::Instant::now();
        let py = self.model.py();
        match self.name {
            RankerName::RankSVM => {
                todo!("RankSVM not implemented yet")
            }
            RankerName::LambdaMart => {
                let kwargs = PyDict::new_bound(py);
                kwargs.set_item("group", group).unwrap();
                self.model
                    .getattr("fit")
                    .unwrap()
                    .call((x, y), Some(&kwargs))
                    .unwrap();
            }
        }
        info!(fitting_time = start_time.elapsed().as_secs_f64());
    }

    pub fn predict(&self, x: &Bound<'py, PyArray2<f64>>) -> Bound<'py, PyArray1<f64>> {
        match self.name {
            RankerName::RankSVM => {
                todo!("RankSVM not implemented yet")
            }
            RankerName::LambdaMart => {
                let result = self.model.getattr("predict").unwrap().call1((x,)).unwrap();
                let f32_result = result.downcast_into::<PyArray1<f32>>().unwrap();
                f32_result.cast(false).unwrap()
            }
        }
    }

    fn construct_ranker(py: Python<'py>, name: RankerName) -> Bound<'py, PyAny> {
        match name {
            RankerName::RankSVM => {
                let svm = py.import_bound("sklearn.svm").unwrap();
                let kwargs = PyDict::new_bound(py);
                kwargs.set_item("C", 1e-6).unwrap();
                kwargs.set_item("loss", "hinge").unwrap();
                kwargs.set_item("max_iter", 9999999).unwrap();
                kwargs.set_item("dual", "auto").unwrap();
                kwargs.set_item("fit_intercept", false).unwrap();

                svm.getattr("LinearSVC")
                    .unwrap()
                    .call((), Some(&kwargs))
                    .unwrap()
            }
            RankerName::LambdaMart => {
                let xgboost = py.import_bound("xgboost").unwrap();
                let kwargs = PyDict::new_bound(py);
                kwargs.set_item("tree_method", "hist").unwrap();
                kwargs.set_item("objective", "rank:ndcg").unwrap();
                kwargs.set_item("lambdarank_pair_method", "mean").unwrap();

                xgboost
                    .getattr("XGBRanker")
                    .unwrap()
                    .call((), Some(&kwargs))
                    .unwrap()
            }
        }
    }

    pub fn pickle(&self, pickle_path: &Path) {
        py_utils::pickle(self.py(), &self.model, pickle_path);
    }

    pub fn unpickle(py: Python<'py>, pickle_path: &Path) -> Self {
        let model = py_utils::unpickle(py, pickle_path);
        Self {
            name: RankerName::LambdaMart,
            model,
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
    use numpy::PyArrayMethods;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_imports() {
        Python::with_gil(|py: Python<'_>| {
            let _ = Ranker::new(py, RankerName::RankSVM);
            let _ = Ranker::new(py, RankerName::LambdaMart);
        })
    }

    #[test]
    fn test_fit_and_predict_for_lambdamart() {
        Python::with_gil(|py| {
            let ranker = Ranker::new(py, RankerName::LambdaMart);
            let x = PyArray2::from_vec2_bound(
                py,
                &[
                    vec![1.0, 1.0],
                    vec![2.0, 2.0],
                    vec![1.2, 1.2],
                    vec![2.2, 2.2],
                    vec![1.3, 1.3],
                ],
            )
            .unwrap();
            let y = PyArray1::from_vec_bound(py, vec![0., 1., 0., 1., 0.]);
            let group = vec![3, 2];
            ranker.fit(&x, &y, &group);

            let x =
                PyArray2::from_vec2_bound(py, &[vec![1.1, 1.1], vec![2.1, 2.1], vec![1.0, 1.0]])
                    .unwrap();
            let y = ranker.predict(&x);
            assert_eq!(y.len().unwrap(), 3);
            let y = y.to_vec().unwrap();
            assert_approx_eq!(y[0], 0.0, 1e-5);
            assert_approx_eq!(y[1], 0.0, 1e-5);
            assert_approx_eq!(y[2], 0.0, 1e-5);
        })
    }
}
