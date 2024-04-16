use crate::search::{Action, ActionSchema, Transition};

/// Struct that represents a partially instantiated action schema.
/// [`PartialAction`] can be viewed as a representation of a set of actions, and
/// hence induce the natural subset relation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn from_action(action: &Action, partial_depth: usize) -> Self {
        assert!(partial_depth <= action.instantiation.len());
        Self {
            index: action.index,
            partial_instantiation: action
                .instantiation
                .iter()
                .take(partial_depth)
                .copied()
                .collect(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn partial_instantiation(&self) -> &Vec<usize> {
        &self.partial_instantiation
    }

    pub fn is_superset_of(&self, other: &PartialAction) -> bool {
        self.index == other.index
            && self
                .partial_instantiation
                .iter()
                .enumerate()
                .all(|(param_index, &object_index)| {
                    other.partial_instantiation.get(param_index) == Some(&object_index)
                })
    }

    pub fn is_subset_of(&self, other: &PartialAction) -> bool {
        other.is_superset_of(self)
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

const NO_PARTIAL: PartialAction = PartialAction {
    index: usize::MAX,
    partial_instantiation: vec![],
};

impl Transition for PartialAction {
    fn no_transition() -> Self {
        NO_PARTIAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_action() {
        let action = Action::new(0, vec![1, 2, 3]);

        // depth 0
        let partial_action = PartialAction::from_action(&action, 0);
        assert_eq!(partial_action.index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![]);

        // depth 1
        let partial_action = PartialAction::from_action(&action, 1);
        assert_eq!(partial_action.index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![1]);

        // depth 2
        let partial_action = PartialAction::from_action(&action, 2);
        assert_eq!(partial_action.index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![1, 2]);

        // depth 3
        let partial_action = PartialAction::from_action(&action, 3);
        assert_eq!(partial_action.index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![1, 2, 3]);
    }

    #[test]
    fn test_subset_relation() {
        let partial = PartialAction::new(0, vec![1, 2]);
        let other = PartialAction::new(0, vec![1, 2, 3]);

        assert!(other.is_subset_of(&partial));
        assert!(partial.is_superset_of(&other));
    }
}
