use crate::learning::models::{Evaluate, WlPalgModel};
use crate::search::{DBState, Heuristic, HeuristicValue, PartialAction, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlPalgHeuristic {
    model: WlPalgModel,
}

impl WlPalgHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WlPalgModel::load(py, saved_model);
        Self { model }
    }
}

impl Heuristic for WlPalgHeuristic {
    type Target = (DBState, PartialAction);

    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(&(state, partial)).into()
    }
}
