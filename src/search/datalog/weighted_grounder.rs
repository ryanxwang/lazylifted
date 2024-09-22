use crate::search::datalog::program::Program;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatalogHeuristicType {
    Hadd,
    Hmax,
    Hff,
}

#[derive(Debug, Clone)]
pub struct WeightedGrounderConfig {
    pub heuristic_type: DatalogHeuristicType,
}

#[derive(Debug)]
pub struct WeightedGrounder {
    config: WeightedGrounderConfig,
}

impl WeightedGrounder {
    pub fn new(_program: &Program, _config: WeightedGrounderConfig) -> Self {
        todo!("Implement WeightedGrounder::new");
    }
}
