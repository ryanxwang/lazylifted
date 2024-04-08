use crate::learning::models::{Evaluate, WLILGModel};
use crate::search::{DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::PathBuf;

pub struct WLILGHeuristic {
    model: WLILGModel,
}

impl WLILGHeuristic {
    pub fn load(saved_model: &PathBuf) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WLILGModel::load(py, saved_model);
        Self { model }
    }
}

impl Heuristic for WLILGHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(&[state])[0].into()
    }
}
