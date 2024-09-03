use crate::learning::graphs::IlgCompiler;
use crate::learning::models::{Evaluate, WlModel};
use crate::search::{DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlStateHeuristic {
    model: WlModel,
    // Since ilg is the only graph for states, we just hardcode it here
    compiler: Option<IlgCompiler>,
}

/// A heuristic that uses the WL-ILG model to evaluate states.
impl WlStateHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WlModel::load(py, saved_model);
        Self {
            model,
            compiler: None,
        }
    }
}

impl Heuristic<DBState> for WlStateHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        if self.compiler.is_none() {
            self.compiler = Some(IlgCompiler::new(task));
        }
        let graph = self.compiler.as_ref().unwrap().compile(state, None);
        self.model.evaluate(graph, None).into()
    }
}
