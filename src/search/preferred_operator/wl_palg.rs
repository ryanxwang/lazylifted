use crate::learning::models::{Evaluate, WLPALGModel};
use crate::search::{Action, ActionSchema, DBState, PreferredOperator, Task};
use pyo3::Python;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct WLPALGPrefOp {
    model: WLPALGModel,
}

impl WLPALGPrefOp {
    pub fn load(saved_model: &PathBuf) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WLPALGModel::load(py, saved_model);
        Self { model }
    }
}

impl PreferredOperator for WLPALGPrefOp {
    fn preferred_operators(
        &mut self,
        state: &DBState,
        task: &Task,
        actions: &[Action],
    ) -> Vec<bool> {
        self.model.set_evaluating_task(task);
        let applicable_schemas: HashSet<usize> =
            actions.iter().map(|action| action.index).collect();
        let applicable_schemas: Vec<&ActionSchema> = task
            .action_schemas
            .iter()
            .filter_map(|schema| {
                if applicable_schemas.contains(&schema.index) {
                    Some(schema)
                } else {
                    None
                }
            })
            .collect();

        let scores = self.model.evaluate_batch(
            &applicable_schemas
                .iter()
                .map(|&schema| (state, schema))
                .collect::<Vec<_>>(),
        );
        let chosen_schema_index = applicable_schemas[scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0]
            .index;
        actions
            .iter()
            .map(|action| action.index == chosen_schema_index)
            .collect()
    }
}
