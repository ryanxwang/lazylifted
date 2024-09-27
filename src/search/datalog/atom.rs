use std::{collections::HashSet, fmt::Display, hash::Hash};

use crate::search::{
    datalog::{arguments::Arguments, term::Term},
    ActionSchema, AtomSchema, SchemaArgument,
};

#[derive(Debug, Clone)]
pub struct Atom {
    arguments: Arguments,
    predicate_index: usize,
    // An artificial predicate is a predicate that is not present in the
    // original task
    is_artificial_predicate: bool,
}

impl Atom {
    pub fn new(
        arguments: Arguments,
        predicate_index: usize,
        is_artificial_predicate: bool,
    ) -> Self {
        Self {
            arguments,
            predicate_index,
            is_artificial_predicate,
        }
    }

    pub fn new_from_atom_schema(atom: &AtomSchema) -> Self {
        let arguments = Arguments::new(
            atom.arguments()
                .iter()
                .map(|schema_argument| match schema_argument {
                    SchemaArgument::Constant(index) => Term::new_object(*index),
                    SchemaArgument::Free(index) => Term::new_variable(*index),
                })
                .collect(),
        );

        Self::new(arguments, atom.predicate_index(), false)
    }

    pub fn new_from_action_schema(action_schema: &ActionSchema, predicate_index: usize) -> Self {
        let arguments = Arguments::new(
            action_schema
                .parameters()
                .iter()
                .map(|schema_parameter| Term::new_variable(schema_parameter.index()))
                .collect(),
        );

        Self::new(arguments, predicate_index, true)
    }

    pub fn arguments(&self) -> &Arguments {
        &self.arguments
    }

    pub fn predicate_index(&self) -> usize {
        self.predicate_index
    }

    pub fn is_artificial_predicate(&self) -> bool {
        self.is_artificial_predicate
    }

    pub fn shares_variable_with(&self, other: &Self) -> bool {
        self.arguments.iter().any(|term| {
            term.is_variable()
                && other
                    .arguments
                    .iter()
                    .any(|other_term| other_term.is_variable() && term == other_term)
        })
    }

    pub fn variables(&self) -> Vec<usize> {
        self.arguments
            .iter()
            .filter_map(|term| {
                if term.is_variable() {
                    Some(term.index())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn variables_set(&self) -> HashSet<usize> {
        self.variables().into_iter().collect()
    }

    pub fn is_variable_unique(&self) -> bool {
        self.variables().len() == self.variables_set().len()
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.predicate_index == other.predicate_index && self.arguments == other.arguments
    }
}

impl Eq for Atom {}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.predicate_index, self.arguments)
    }
}

impl Hash for Atom {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.predicate_index.hash(state);
        self.arguments.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serial_test::serial;

    // Serial is needed to make sure the global counter is not modified by other
    // tests

    #[test]
    #[serial]
    fn test_atom_new() {
        let arguments = Arguments::new(vec![Term::new_variable(0), Term::new_object(1)]);
        let atom = Atom::new(arguments, 0, false);
        assert_eq!(atom.arguments().len(), 2);
        assert_eq!(atom.predicate_index(), 0);
        assert!(!atom.is_artificial_predicate());
    }

    #[test]
    #[serial]
    fn test_atom_new_from_atom_schema() {
        let atom_schema = AtomSchema::new(
            0,
            vec![SchemaArgument::Free(0), SchemaArgument::Constant(1)],
        );
        let atom = Atom::new_from_atom_schema(&atom_schema);
        assert_eq!(atom.arguments().len(), 2);
        assert_eq!(atom.predicate_index(), 0);
        assert!(!atom.is_artificial_predicate());
        assert_eq!(atom.arguments()[0], Term::new_variable(0));
        assert_eq!(atom.arguments()[1], Term::new_object(1));
    }

    #[test]
    fn test_atom_shares_variable_with() {
        let arguments1 = Arguments::new(vec![Term::new_variable(0), Term::new_object(1)]);
        let atom1 = Atom::new(arguments1, 0, false);

        let arguments2 = Arguments::new(vec![Term::new_variable(0), Term::new_object(1)]);
        let atom2 = Atom::new(arguments2, 0, false);

        assert!(atom1.shares_variable_with(&atom2));

        // Different variables should lead to false
        let arguments3 = Arguments::new(vec![Term::new_variable(1), Term::new_object(1)]);
        let atom3 = Atom::new(arguments3, 0, false);
        assert!(!atom1.shares_variable_with(&atom3));

        // No variables should lead to false, even if same object
        let arguments4 = Arguments::new(vec![Term::new_object(1), Term::new_object(1)]);
        let atom4 = Atom::new(arguments4, 0, false);
        assert!(!atom1.shares_variable_with(&atom4));
    }
}
