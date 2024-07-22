use crate::learning::models::{Evaluate, SchemaDecomposedModel};
use crate::search::{states::SchemaDecomposedState, DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlSchemaDecomposedHeuristic {
    model: SchemaDecomposedModel,
}

impl WlSchemaDecomposedHeuristic {
    pub fn load(saved_mode: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = SchemaDecomposedModel::load(py, saved_mode);
        Self { model }
    }
}

impl Heuristic<SchemaDecomposedState<DBState>> for WlSchemaDecomposedHeuristic {
    fn evaluate(&mut self, state: &SchemaDecomposedState<DBState>, task: &Task) -> HeuristicValue {
        self.model.set_evaluating_task(task);
        self.model.evaluate(state).into()
    }
}
