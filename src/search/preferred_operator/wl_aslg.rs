use crate::learning::models::{Evaluate, WLASLGModel};
use crate::search::{Action, DBState, PreferredOperator, Task};
use pyo3::Python;
use std::path::PathBuf;

pub struct WLASLGPrefOp {
    model: WLASLGModel,
}

impl WLASLGPrefOp {
    pub fn load(saved_model: &PathBuf) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WLASLGModel::load(py, saved_model);
        Self { model }
    }
}

impl PreferredOperator for WLASLGPrefOp {
    fn preferred_operators(
        &mut self,
        state: &DBState,
        task: &Task,
        actions: &[Action],
    ) -> Vec<bool> {
        self.model.set_evaluating_task(task);
        let scores = self.model.evaluate_batch(
            &task
                .action_schemas
                .iter()
                .map(|schema| (state, schema))
                .collect::<Vec<_>>(),
        );
        let chosen_schema_index = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        actions
            .iter()
            .map(|action| action.index == chosen_schema_index)
            .collect()
    }
}
