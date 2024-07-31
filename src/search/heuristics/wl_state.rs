use crate::learning::models::{Evaluate, StateSpaceModel};
use crate::search::{DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlStateHeuristic {
    model: StateSpaceModel,
}

/// A heuristic that uses the WL-ILG model to evaluate states.
impl WlStateHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = StateSpaceModel::load(py, saved_model);
        Self { model }
    }
}

impl Heuristic<DBState> for WlStateHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(state).into()
    }
}
