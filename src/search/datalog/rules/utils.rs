use crate::search::datalog::{atom::Atom, term::Term};
use std::collections::HashMap;
use strum_macros::EnumIs;

/// This is a map to keep track of the position of variables in the effect of a
/// rule for easy access.
#[derive(Debug, Clone)]
pub struct VariablePositionInEffect {
    mapping: HashMap<Term, usize>,
}

impl VariablePositionInEffect {
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

#[derive(Debug, Clone, EnumIs)]
pub enum VariablePositionInBody {
    /// A [`Direct`](VariablePositionInBody::Direct) entry means that we can
    /// find the variable in the condition at the
    /// [`condition_index`](VariablePositionInBody::Direct::condition_index),
    /// and it is the
    /// [`variable_index`](VariablePositionInBody::Direct::variable_index)-th
    /// argument of the condition.
    Direct {
        condition_index: usize,
        variable_index: usize,
    },
    /// An [`Indirect`](VariablePositionInBody::Indirect) entry means that we
    /// can find the variable in the [`VariableSource`] of the achiever rule of
    /// the condition at the
    /// [`condition_index`](VariablePositionInBody::Indirect::condition_index),
    /// with the table index of the variable being
    /// [`table_index`](VariablePositionInBody::Indirect::table_index).
    Indirect {
        condition_index: usize,
        table_index: usize,
    },
}

impl VariablePositionInBody {
    pub fn condition_index(&self) -> usize {
        match self {
            Self::Direct {
                condition_index, ..
            } => *condition_index,
            Self::Indirect {
                condition_index, ..
            } => *condition_index,
        }
    }

    pub fn set_condition_index(&mut self, condition_index: usize) {
        match self {
            Self::Direct {
                condition_index: i, ..
            } => *i = condition_index,
            Self::Indirect {
                condition_index: i, ..
            } => *i = condition_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableSource {
    /// A table that maps variables to their positions in the conditions of a
    /// rule.
    table: Vec<VariablePositionInBody>,
    variable_index_to_table_index: HashMap<usize, usize>,
    table_index_to_variable_index: HashMap<usize, usize>,
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
        let mut variable_index_to_table_index = HashMap::new();
        let mut table_index_to_variable_index = HashMap::new();
        for term in all_variables {
            assert!(term.is_variable());
            let table_index = table.len();
            let variable_index = term.index();

            variable_index_to_table_index.insert(variable_index, table_index);
            table_index_to_variable_index.insert(table_index, variable_index);

            let position = conditions.iter().enumerate().find_map(|(i, condition)| {
                condition
                    .arguments()
                    .iter()
                    .position(|t| t == &term)
                    .map(|j| (i, j))
            });
            match position {
                Some(position) => {
                    table.push(VariablePositionInBody::Direct {
                        condition_index: position.0,
                        variable_index: position.1,
                    });
                }
                None => {
                    panic!("Variable {:?} not found in the conditions, this probably could be handle, but isn't implemented yet", term);
                }
            }
        }

        Self {
            table,
            variable_index_to_table_index,
            table_index_to_variable_index,
        }
    }

    pub fn table(&self) -> &[VariablePositionInBody] {
        &self.table
    }

    pub fn get_variable_index_from_table_index(&self, table_index: usize) -> usize {
        *self
            .table_index_to_variable_index
            .get(&table_index)
            .unwrap()
    }

    pub fn get_table_index_from_variable_index(&self, variable_index: usize) -> usize {
        *self
            .variable_index_to_table_index
            .get(&variable_index)
            .unwrap()
    }

    pub fn is_variable_in_body(&self, term: &Term) -> bool {
        assert!(term.is_variable());
        !self.table[self.variable_index_to_table_index[&term.index()]].is_direct()
    }

    /// Update the condition indices of the entries in the table.
    pub fn update_condition_indices(&mut self, condition_indices: &HashMap<usize, usize>) {
        for position in self.table.iter_mut() {
            position.set_condition_index(condition_indices[&position.condition_index()]);
        }
    }

    /// Update all the entries in the table that have the condition index
    /// `condition_index` to point to the new source.
    pub fn update_entries_with_new_source(
        &mut self,
        condition_index: usize,
        new_source: &VariableSource,
    ) {
        for table_index in 0..self.table.len() {
            if self.table[table_index].condition_index() == condition_index {
                assert!(self.table[table_index].is_direct());

                let variable_index = self.get_variable_index_from_table_index(table_index);
                let indirect_variable_index =
                    new_source.get_table_index_from_variable_index(variable_index);

                self.table[table_index] = VariablePositionInBody::Indirect {
                    condition_index,
                    table_index: indirect_variable_index,
                };
            }
        }
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

        let variable_position_map = VariablePositionInEffect::new(&effect);
        assert!(!variable_position_map.has_variable(&Term::new_object(0)),);
        assert!(variable_position_map.has_variable(&Term::new_variable(3)),);
        assert_eq!(variable_position_map.get(&Term::new_object(0)), None);
        assert_eq!(variable_position_map.get(&Term::new_variable(3)), Some(1));
    }

    // TODO-soon: Add tests for VariableSource
}
