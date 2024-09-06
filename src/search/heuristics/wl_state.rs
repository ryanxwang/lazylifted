use crate::learning::graphs::{Compiler, StateCompilerConfig};
use crate::learning::models::{Evaluate, WlModel};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, Heuristic, HeuristicValue, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlStateHeuristic {
    model: WlModel,
    successor_generator_name: SuccessorGeneratorName,
    compiler_name: StateCompilerConfig,
    compiler: Option<Box<dyn Compiler<DBState>>>,
}

/// A heuristic that uses the WL-ILG model to evaluate states.
impl WlStateHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WlModel::load(py, saved_model);
        let successor_generator_name = model.successor_generator_name();

        match model.state_compiler_name() {
            Some(compiler_name) => Self {
                model,
                compiler_name,
                compiler: None,
                successor_generator_name,
            },
            None => panic!("Model does not specify which graph compiler to use"),
        }
    }
}

impl Heuristic<DBState> for WlStateHeuristic {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        if self.compiler.is_none() {
            self.compiler = Some(
                self.compiler_name
                    .create(task, self.successor_generator_name),
            );
        }
        let graph = self.compiler.as_ref().unwrap().compile(state, None);
        self.model.evaluate(graph, None).into()
    }
}
