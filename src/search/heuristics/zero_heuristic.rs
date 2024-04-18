use crate::search::{Heuristic, HeuristicValue, Task};

#[derive(Clone, Debug, Default)]
pub struct ZeroHeuristic {}

impl ZeroHeuristic {
    pub fn new() -> Self {
        ZeroHeuristic {}
    }
}

impl<T> Heuristic<T> for ZeroHeuristic {
    fn evaluate(&mut self, _state: &T, _task: &Task) -> HeuristicValue {
        (0.).into()
    }
}
