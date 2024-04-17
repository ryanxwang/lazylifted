//! A heuristic that uses the WL-ILG model to evaluate states. Note that we
//! intentionally do not batch evaluate states, as batch evaluations seem to
//! result in worse performance (10x), possibly due to worse cache locality.
// TODO investigate cache performance

use crate::learning::models::{Evaluate, WlIlgModel};
use crate::search::{DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlIlgHeuristic {
    model: WlIlgModel,
}

/// A heuristic that uses the WL-ILG model to evaluate states.
impl WlIlgHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WlIlgModel::load(py, saved_model);
        Self { model }
    }
}

impl Heuristic for WlIlgHeuristic {
    type Target = DBState;

    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(state).into()
    }
}
