use crate::search::{Action, ActionSchema};

/// Struct that represents a partially instantiated action schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialAction {
    /// The action schema index.
    index: usize,
    partial_instantiation: Vec<usize>,
}

impl PartialAction {
    pub fn new(index: usize, partial_instantiation: Vec<usize>) -> Self {
        Self {
            index,
            partial_instantiation,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn partial_instantiation(&self) -> &Vec<usize> {
        &self.partial_instantiation
    }

    pub fn is_subset_of(&self, other: &PartialAction) -> bool {
        self.index == other.index
            && self
                .partial_instantiation
                .iter()
                .enumerate()
                .all(|(param_index, &object_index)| {
                    other.partial_instantiation.get(param_index) == Some(&object_index)
                })
    }

    pub fn is_superset_of(&self, other: &PartialAction) -> bool {
        other.is_subset_of(self)
    }
}

impl From<Action> for PartialAction {
    fn from(action: Action) -> Self {
        Self {
            index: action.index,
            partial_instantiation: action.instantiation,
        }
    }
}

impl From<ActionSchema> for PartialAction {
    fn from(action_schema: ActionSchema) -> Self {
        Self {
            index: action_schema.index,
            partial_instantiation: vec![],
        }
    }
}
