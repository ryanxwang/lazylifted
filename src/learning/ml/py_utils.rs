use std::path::Path;

use pyo3::prelude::*;

pub fn get_regression_model(py: Python) -> Bound<'_, PyAny> {
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/learning/ml/python/regression_model.py"
    ));
    PyModule::from_code_bound(py, code, "regression_model", "regression_model")
        .unwrap()
        .getattr("RegressionModel")
        .unwrap()
}

pub fn get_ranking_model(py: Python) -> Bound<'_, PyAny> {
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/learning/ml/python/ranking_model.py"
    ));
    PyModule::from_code_bound(py, code, "ranking_model", "ranking_model")
        .unwrap()
        .getattr("RankingModel")
        .unwrap()
}

pub fn pickle<'py>(py: Python<'py>, object: &Bound<'py, PyAny>, path: &Path) {
    let pickle = py.import_bound("pickle").unwrap();
    let file = py
        .import_bound("builtins")
        .unwrap()
        .getattr("open")
        .unwrap()
        .call1((path.to_str().unwrap(), "wb"))
        .unwrap();
    pickle
        .getattr("dump")
        .unwrap()
        .call1((object, &file))
        .unwrap();
    file.getattr("close").unwrap().call0().unwrap();
}

pub fn unpickle<'py>(py: Python<'py>, pickle_path: &Path) -> Bound<'py, PyAny> {
    let _regression_model = get_regression_model(py);
    let _ranking_model = get_ranking_model(py);

    let pickle = py.import_bound("pickle").unwrap();
    let file = py
        .import_bound("builtins")
        .unwrap()
        .getattr("open")
        .unwrap()
        .call1((pickle_path.to_str().unwrap(), "rb"))
        .unwrap();
    let object = pickle.getattr("load").unwrap().call1((&file,)).unwrap();
    file.getattr("close").unwrap().call0().unwrap();
    object
}
