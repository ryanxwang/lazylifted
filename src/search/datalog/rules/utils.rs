use crate::search::datalog::{atom::Atom, term::Term};
use std::collections::HashMap;

/// This is a map to keep track of the position of variables in the effect of a
/// rule for easy access.
#[derive(Debug, Clone)]
pub struct VariablePositionMap {
    mapping: HashMap<Term, usize>,
}

impl VariablePositionMap {
    pub fn new(effect: &Atom) -> Self {
        let mut mapping = HashMap::new();
        for (i, term) in effect.arguments().iter().enumerate() {
            if term.is_variable() {
                mapping.insert(term.to_owned(), i);
            }
        }
        Self { mapping }
    }

    pub fn has_variable(&self, term: &Term) -> bool {
        self.mapping.contains_key(term)
    }

    pub fn get(&self, term: &Term) -> Option<usize> {
        self.mapping.get(term).copied()
    }
}

#[derive(Debug, Clone)]
pub struct VariableSource {
    /// A table that maps variables to their positions in the conditions of a
    /// rule. Each entry in the table is a tuple of the form (i, j), where i is
    /// the index of the condition in the conditions vector and j is the index
    /// of the variable in the arguments of the condition.
    table: Vec<Option<(usize, usize)>>,
    term_index_to_table_index: HashMap<usize, usize>,
    table_index_to_term_index: HashMap<usize, usize>,
}

impl VariableSource {
    pub fn new(effect: &Atom, conditions: &[Atom]) -> Self {
        let mut all_variables = vec![];
        for term in effect.arguments() {
            if term.is_variable() {
                all_variables.push(term.to_owned());
            }
        }
        for term in conditions.iter().flat_map(|atom| atom.arguments()) {
            if term.is_variable() {
                all_variables.push(term.to_owned());
            }
        }

        let mut table = vec![];
        let mut term_index_to_table_index = HashMap::new();
        let mut table_index_to_term_index = HashMap::new();
        for term in all_variables {
            assert!(term.is_variable());
            let table_index = table.len();
            let term_index = term.index;

            term_index_to_table_index.insert(term_index, table_index);
            table_index_to_term_index.insert(table_index, term_index);

            let position = conditions.iter().enumerate().find_map(|(i, condition)| {
                condition
                    .arguments()
                    .iter()
                    .position(|t| t == &term)
                    .map(|j| (i, j))
            });
            table.push(position);
        }

        Self {
            table,
            term_index_to_table_index,
            table_index_to_term_index,
        }
    }

    pub fn table(&self) -> &[Option<(usize, usize)>] {
        &self.table
    }

    pub fn get_term_index_from_table_index(&self, table_index: usize) -> usize {
        *self.table_index_to_term_index.get(&table_index).unwrap()
    }

    pub fn get_table_index_from_term_index(&self, term_index: usize) -> usize {
        *self.term_index_to_table_index.get(&term_index).unwrap()
    }

    pub fn is_variable_in_body(&self, term: &Term) -> bool {
        assert!(term.is_variable());
        self.table[self.term_index_to_table_index[&term.index]].is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::datalog::arguments::Arguments;

    #[test]
    fn test_variable_position_map() {
        let effect = Atom::new(
            Arguments::new(vec![Term::new_object(0), Term::new_variable(3)]),
            0,
            false,
        );

        let variable_position_map = VariablePositionMap::new(&effect);
        assert!(!variable_position_map.has_variable(&Term::new_object(0)),);
        assert!(variable_position_map.has_variable(&Term::new_variable(3)),);
        assert_eq!(variable_position_map.get(&Term::new_object(0)), None);
        assert_eq!(variable_position_map.get(&Term::new_variable(3)), Some(1));
    }

    // TODO-soon: Add tests for VariableSource
}
