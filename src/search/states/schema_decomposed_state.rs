use crate::search::{states::SparsePackedState, Action, Transition};

/// A [`SchemaDecomposedState`] is a search state where we first decide which
/// schema then which action to apply. It is in-between the normal planning
/// search state and the partial action search state.
#[derive(Hash, Debug, Clone, Eq, PartialEq)]
pub struct SchemaDecomposedState {
    state: SparsePackedState,
    schema: Option<usize>,
}

impl SchemaDecomposedState {
    pub fn new(state: SparsePackedState, schema: Option<usize>) -> Self {
        Self { state, schema }
    }

    pub fn without_schema(state: SparsePackedState) -> Self {
        Self {
            state,
            schema: None,
        }
    }

    pub fn with_schema(state: SparsePackedState, schema: usize) -> Self {
        Self {
            state,
            schema: Some(schema),
        }
    }

    pub fn state(&self) -> &SparsePackedState {
        &self.state
    }

    pub fn schema(&self) -> Option<usize> {
        self.schema
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
