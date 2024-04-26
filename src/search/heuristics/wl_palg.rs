use crate::learning::models::{Evaluate, PartialActionModel};
use crate::search::{DBState, Heuristic, HeuristicValue, PartialAction, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlPalgHeuristic {
    model: PartialActionModel,
}

impl WlPalgHeuristic {
    pub fn load(config: &Path, saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = PartialActionModel::load(py, config, saved_model);
        Self { model }
    }
}

impl Heuristic<(DBState, PartialAction)> for WlPalgHeuristic {
    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(&(state, partial)).into()
    }
}
