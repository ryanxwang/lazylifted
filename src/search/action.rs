/// Action struct that represents an instantiated action schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The action schema index.
    pub index: usize,
    /// List of object indices that instantiate the action schema.
    pub instantiation: Vec<usize>,
}

impl Action {
    fn new(index: usize, instantiation: Vec<usize>) -> Self {
        Self {
            index,
            instantiation,
        }
    }
}

pub const NO_ACTION: Action = Action {
    index: usize::MAX,
    instantiation: vec![],
};
