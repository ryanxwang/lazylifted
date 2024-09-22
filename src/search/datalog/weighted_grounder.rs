use crate::search::datalog::program::Program;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatalogHeuristicType {
    #[allow(dead_code)]
    Hadd,
    Hmax,
    #[allow(dead_code)]
    Hff,
}

#[derive(Debug, Clone)]
pub struct WeightedGrounderConfig {
    #[allow(dead_code)]
    pub heuristic_type: DatalogHeuristicType,
}

#[derive(Debug)]
pub struct WeightedGrounder {
    #[allow(dead_code)]
    config: WeightedGrounderConfig,
}

impl WeightedGrounder {
    pub fn new(_program: &Program, _config: WeightedGrounderConfig) -> Self {
        todo!("Implement WeightedGrounder::new");
    }
}
