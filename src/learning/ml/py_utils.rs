use std::path::Path;

use pyo3::{types::PyAnyMethods, Bound, PyAny, Python};

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
