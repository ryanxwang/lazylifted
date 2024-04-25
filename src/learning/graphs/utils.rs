use crate::search::SchemaArgument;
use serde::{Deserialize, Serialize};

/// A representation of a schema predication, which is basically just a
/// [`crate::search::SchemaAtom`] without the
/// [`crate::search::SchemaAtom::negated`] field.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaPred {
    predicate_index: usize,
    arguments: Vec<SchemaArgument>,
}

impl SchemaPred {
    pub fn new(predicate_index: usize, arguments: Vec<SchemaArgument>) -> Self {
        Self {
            predicate_index,
            arguments,
        }
    }

    pub fn predicate_index(&self) -> usize {
        self.predicate_index
    }

    pub fn arguments(&self) -> &[SchemaArgument] {
        &self.arguments
    }
}
