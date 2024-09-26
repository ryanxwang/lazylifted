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

    pub fn negative_auxiliary_predicate(&self, index: usize) -> Self {
        let mut negative_predicate = self.clone();
        negative_predicate.name = Name::new(format!("not@{}", self.name));
        negative_predicate.index = index;
        negative_predicate
    }

    pub fn mark_as_static(&mut self) {
        self.is_static = true;
    }
}
