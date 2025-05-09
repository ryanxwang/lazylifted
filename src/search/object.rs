use crate::parsed_types::{Name, Typed};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub name: Name,
    pub index: usize,
    pub types: Vec<usize>,
}

impl Object {
    pub fn new(index: usize, object: &Typed<Name>, type_table: &HashMap<Name, usize>) -> Self {
        let types = object
            .type_()
            .clone()
            .into_iter()
            .map(|t| {
                *type_table
                    .get(t.name())
                    .expect("Object type not found in domain type table.")
            })
            .collect();
        Self {
            name: object.value().clone(),
            index,
            types,
        }
    }
}
