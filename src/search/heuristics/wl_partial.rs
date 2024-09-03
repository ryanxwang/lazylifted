use crate::learning::graphs::{PartialActionCompiler, PartialActionCompilerName};
use crate::learning::models::{Evaluate, WlModel};
use crate::search::successor_generators::SuccessorGeneratorName;
use crate::search::{DBState, Heuristic, HeuristicValue, PartialAction, Task};
use pyo3::Python;
use std::path::Path;

#[derive(Debug)]
pub struct WlPartialHeuristic {
    model: WlModel,
    compiler_name: PartialActionCompilerName,
    successor_generator_name: SuccessorGeneratorName,
    compiler: Option<Box<dyn PartialActionCompiler>>,
}

impl WlPartialHeuristic {
    pub fn load(saved_model: &Path) -> Self {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = WlModel::load(py, saved_model);
        let successor_generator_name = model.successor_generator_name();

        match model.compiler_name() {
            Some(compiler_name) => Self {
                model,
                compiler: None,
                compiler_name,
                successor_generator_name,
            },
            None => panic!("Model does not specify which graph compiler to use"),
        }
    }
}

impl Heuristic<(DBState, PartialAction)> for WlPartialHeuristic {
    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        if self.compiler.is_none() {
            self.compiler = Some(
                self.compiler_name
                    .create(task, self.successor_generator_name),
            );
        }
        let graph = self
            .compiler
            .as_ref()
            .unwrap()
            .compile(state, partial, None);
        self.model.evaluate(graph, Some(partial.group_id())).into()
    }
}
