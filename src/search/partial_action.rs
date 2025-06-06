use std::collections::HashSet;

use crate::search::{Action, ActionSchema, Atom, Negatable, Task, Transition};

/// Struct that represents a partially instantiated action schema.
/// [`PartialAction`] can be viewed as a representation of a set of actions, and
/// hence induce the natural subset relation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartialAction {
    schema_index: usize,
    // TODO-soon: this probably could be a smallvec
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

    #[inline(always)]
    pub fn depth(&self) -> usize {
        self.partial_instantiation.len()
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

    // TODO-soon test
    pub fn get_partial_effects(
        &self,
        action_schema: &ActionSchema,
        applicable_actions: &[Action],
    ) -> PartialEffects {
        if *self == NO_PARTIAL {
            return PartialEffects {
                unavoidable_effects: HashSet::new(),
                optional_effects: HashSet::new(),
            };
        }
        assert!(self.schema_index == action_schema.index());

        let action_effects: Vec<HashSet<Negatable<Atom>>> = applicable_actions
            .iter()
            .map(|action| action_schema.ground_effects(action).into_iter().collect())
            .collect::<Vec<_>>();

        if action_effects.is_empty() {
            return PartialEffects {
                unavoidable_effects: HashSet::new(),
                optional_effects: HashSet::new(),
            };
        }

        // the intersection of all the action effects are unavoidable
        let unavoidable_effects: HashSet<Negatable<Atom>> = action_effects
            .iter()
            .fold(action_effects[0].clone(), |acc, effects| {
                acc.intersection(effects).cloned().collect()
            });

        // the union of all the action effects, minus the unavoidable effects,
        // are optional
        let optional_effects: HashSet<Negatable<Atom>> = action_effects
            .iter()
            .fold(action_effects[0].clone(), |acc, effects| {
                acc.union(effects).cloned().collect()
            })
            .difference(&unavoidable_effects)
            .cloned()
            .collect();

        PartialEffects {
            unavoidable_effects,
            optional_effects,
        }
    }

    pub fn human_readable(&self, task: &Task) -> String {
        let action_schema = task.action_schemas()[self.schema_index].clone();
        let parameters = action_schema.parameters();

        format!(
            "({} {})",
            action_schema.name(),
            parameters
                .iter()
                .enumerate()
                .map(|param| {
                    let object_index = self.partial_instantiation.get(param.0).copied();
                    match object_index {
                        Some(index) => task.objects[index].name.to_string(),
                        None => "_".to_string(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        )
    }

    /// The group ID of a partial action describes which partial actions belong
    /// in the same feature space. Here we consider partial actions with the
    /// same schema and the same depth to be in the same group, so the group ID
    /// is effectively a hash of the schema index and the depth.
    pub fn group_id(&self) -> usize {
        // As primitive as it gets
        self.schema_index * 100 + self.partial_instantiation.len()
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

#[derive(Debug)]
/// The effects of a partial action, split into unavoidable (any grounding of
/// this partial in the current state yields this effect) and optional effects.
/// See [`PartialAction::get_partial_effects`].
pub struct PartialEffects {
    pub unavoidable_effects: HashSet<Negatable<Atom>>,
    pub optional_effects: HashSet<Negatable<Atom>>,
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
        // need to specify the type of the empty vector to avoid conflict with serde_json
        assert_eq!(*partial_action.partial_instantiation(), Vec::<usize>::new());

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
