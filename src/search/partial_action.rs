use crate::search::{Action, ActionSchema, Task, Transition};

/// Struct that represents a partially instantiated action schema.
/// [`PartialAction`] can be viewed as a representation of a set of actions, and
/// hence induce the natural subset relation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartialAction {
    schema_index: usize,
    partial_instantiation: Vec<usize>,
}

/// This should only be used as the action associated with the initial node of
/// the search space.
pub const NO_PARTIAL: PartialAction = PartialAction {
    schema_index: usize::MAX,
    partial_instantiation: vec![],
};

impl PartialAction {
    pub fn new(index: usize, partial_instantiation: Vec<usize>) -> Self {
        Self {
            schema_index: index,
            partial_instantiation,
        }
    }

    pub fn from_action(action: &Action, partial_depth: usize) -> Self {
        assert!(partial_depth <= action.instantiation.len());
        Self {
            schema_index: action.index,
            partial_instantiation: action
                .instantiation
                .iter()
                .take(partial_depth)
                .copied()
                .collect(),
        }
    }

    #[inline(always)]
    pub fn schema_index(&self) -> usize {
        self.schema_index
    }

    #[inline(always)]
    pub fn partial_instantiation(&self) -> &[usize] {
        self.partial_instantiation.as_slice()
    }

    pub fn is_complete(&self, task: &Task) -> bool {
        self.partial_instantiation.len()
            == task.action_schemas()[self.schema_index].parameters().len()
    }

    pub fn is_superset_of(&self, other: &PartialAction) -> bool {
        self.schema_index == other.schema_index
            && self
                .partial_instantiation
                .iter()
                .enumerate()
                .all(|(param_index, &object_index)| {
                    other.partial_instantiation.get(param_index) == Some(&object_index)
                })
    }

    pub fn is_superset_of_action(&self, action: &Action) -> bool {
        self.schema_index == action.index
            && self
                .partial_instantiation
                .iter()
                .enumerate()
                .all(|(param_index, &object_index)| {
                    action.instantiation.get(param_index) == Some(&object_index)
                })
    }

    pub fn is_subset_of(&self, other: &PartialAction) -> bool {
        other.is_superset_of(self)
    }

    pub fn add_instantiation(&self, object_index: usize) -> Self {
        let mut new_instantiation = self.partial_instantiation.clone();
        new_instantiation.push(object_index);
        Self {
            schema_index: self.schema_index,
            partial_instantiation: new_instantiation,
        }
    }
}

impl From<Action> for PartialAction {
    fn from(action: Action) -> Self {
        Self {
            schema_index: action.index,
            partial_instantiation: action.instantiation,
        }
    }
}

impl From<ActionSchema> for PartialAction {
    fn from(action_schema: ActionSchema) -> Self {
        Self {
            schema_index: action_schema.index(),
            partial_instantiation: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PartialActionDiff {
    Schema(usize),
    Instantiation(usize),
}

/// This should be used only as a placeholder for the parent action of the
/// initial node
const NO_PARTIAL_DIFF: PartialActionDiff = PartialActionDiff::Schema(usize::MAX);

impl Transition for PartialActionDiff {
    fn no_transition() -> Self {
        NO_PARTIAL_DIFF
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
        assert_eq!(partial_action.schema_index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![]);

        // depth 1
        let partial_action = PartialAction::from_action(&action, 1);
        assert_eq!(partial_action.schema_index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![1]);

        // depth 2
        let partial_action = PartialAction::from_action(&action, 2);
        assert_eq!(partial_action.schema_index(), 0);
        assert_eq!(*partial_action.partial_instantiation(), vec![1, 2]);

        // depth 3
        let partial_action = PartialAction::from_action(&action, 3);
        assert_eq!(partial_action.schema_index(), 0);
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
