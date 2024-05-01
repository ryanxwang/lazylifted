use std::collections::HashMap;

use crate::parsed_types::{Name, PredicateDefinition};

#[derive(Debug, Clone)]
pub struct Predicate {
    pub name: Name,
    pub index: usize,
    pub arity: usize,
    pub types: Vec<usize>,
    pub is_static: bool,
}

impl Predicate {
    pub fn new(
        index: usize,
        predicate_definition: &PredicateDefinition,
        type_table: &HashMap<Name, usize>,
    ) -> Self {
        let types = predicate_definition
            .variables()
            .iter()
            .map(|x| {
                *type_table
                    .get(
                        x.type_()
                            .get_primitive()
                            .expect("Predicates should have primitive typed arguments.")
                            .name(),
                    )
                    .expect("Predicate argument type not found in domain type table.")
            })
            .collect();

        Self {
            name: predicate_definition.name().clone(),
            index,
            arity: predicate_definition.variables().len(),
            types,
            is_static: false,
        }
    }

    pub fn mark_as_static(&mut self) {
        self.is_static = true;
    }
}
