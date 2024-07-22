use crate::search::{Action, Transition};

/// A [`SchemaDecomposedState`] is a search state where we first decide which
/// schema then which action to apply. It is in-between the normal planning
/// search state and the partial action search state.
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct SchemaDecomposedState<S> {
    state: S,
    schema: Option<usize>,
}

impl<S> SchemaDecomposedState<S> {
    pub fn new(state: S, schema: Option<usize>) -> Self {
        Self { state, schema }
    }

    pub fn without_schema(state: S) -> Self {
        Self {
            state,
            schema: None,
        }
    }

    pub fn with_schema(state: S, schema: usize) -> Self {
        Self {
            state,
            schema: Some(schema),
        }
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn schema(&self) -> Option<usize> {
        self.schema
    }

    /// The group ID is used for grouping states when ranking. The group ID of a
    /// schema decomposed state effectively a hash of the schema field.
    pub fn group_id(&self) -> usize {
        self.schema.unwrap_or(usize::MAX)
    }
}

/// A [`SchemaOrInstantiation`] is a union type that can represent the
/// transition between two [`SchemaDecomposedState`]s. It can either be deciding
/// to apply a schema in a state or deciding how to instantiate a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaOrInstantiation {
    Schema(usize),
    Instantiation(Action),
}

/// This should be used only as a placeholder for the parent action of the
/// initial node
const NO_SCHEMA_OR_INSTANTIATION: SchemaOrInstantiation = SchemaOrInstantiation::Schema(usize::MAX);

impl Transition for SchemaOrInstantiation {
    fn no_transition() -> Self {
        NO_SCHEMA_OR_INSTANTIATION
    }
}

impl SchemaOrInstantiation {
    pub fn from_action(action: &Action) -> Vec<Self> {
        vec![
            SchemaOrInstantiation::Schema(action.index),
            SchemaOrInstantiation::Instantiation(action.clone()),
        ]
    }
}
