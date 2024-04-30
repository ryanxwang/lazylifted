use crate::learning::models::{Evaluate, PartialActionModel};
use crate::search::{DBState, Heuristic, HeuristicValue, PartialAction, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlPartialHeuristic {
    model: PartialActionModel,
}

impl WlPartialHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = PartialActionModel::load(py, saved_model);
        Self { model }
    }
}

impl Heuristic<(DBState, PartialAction)> for WlPartialHeuristic {
    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(&(state, partial)).into()
    }
}
