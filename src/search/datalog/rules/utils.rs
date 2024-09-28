use crate::search::datalog::atom::Atom;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use strum_macros::EnumIs;

/// This is a map to keep track of the position of variables in the effect of a
/// rule for easy access.
#[derive(Debug, Clone)]
pub struct VariablePositionInEffect {
    mapping: HashMap<usize, usize>,
}

impl VariablePositionInEffect {
    pub fn new(effect: &Atom) -> Self {
        let mut mapping = HashMap::new();
        for (i, term) in effect.arguments().iter().enumerate() {
            if term.is_variable() {
                mapping.insert(term.index(), i);
            }
        }
        Self { mapping }
    }

    pub fn get(&self, variable_index: usize) -> Option<usize> {
        self.mapping.get(&variable_index).copied()
    }
}

#[derive(Debug, Clone, EnumIs)]
pub enum VariablePositionInBody {
    /// A [`Direct`](VariablePositionInBody::Direct) entry means that we can
    /// find the variable in the condition at the
    /// [`condition_index`](VariablePositionInBody::Direct::condition_index),
    /// and it is the
    /// [`argument_index`](VariablePositionInBody::Direct::argument_index)-th
    /// argument of the condition.
    Direct {
        condition_index: usize,
        argument_index: usize,
    },
    /// An [`Indirect`](VariablePositionInBody::Indirect) entry means that we
    /// can find the variable in the [`VariableSource`] of the achiever rule of
    /// the condition at the
    /// [`condition_index`](VariablePositionInBody::Indirect::condition_index),
    /// with the table index of the variable being
    /// [`table_index`](VariablePositionInBody::Indirect::table_index).
    Indirect {
        condition_index: usize,
        // we probably don't actually need this, knowing the variable index is
        // enough
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

impl Display for VariablePositionInBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct {
                condition_index,
                argument_index,
            } => write!(
                f,
                "Direct {{ condition_index: {}, argument_index: {} }}",
                condition_index, argument_index
            ),
            Self::Indirect {
                condition_index,
                table_index,
            } => write!(
                f,
                "Indirect {{ condition_index: {}, table_index: {} }}",
                condition_index, table_index
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableSource {
    /// A table that maps variables to their positions in the conditions of a
    /// rule.
    pub(super) table: Vec<VariablePositionInBody>,
    pub(super) variable_index_to_table_index: HashMap<usize, usize>,
    pub(super) table_index_to_variable_index: HashMap<usize, usize>,
}

impl VariableSource {
    pub fn new(effect: &Atom, conditions: &[Atom]) -> Self {
        let mut all_variables = HashSet::new();
        for term in effect.arguments() {
            if term.is_variable() {
                all_variables.insert(term.to_owned());
            }
        }
        for term in conditions.iter().flat_map(|atom| atom.arguments()) {
            if term.is_variable() {
                all_variables.insert(term.to_owned());
            }
        }
        // Sort the variables to ensure that the order is deterministic.
        let all_variables: Vec<_> = all_variables.into_iter().sorted().collect();

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
                        argument_index: position.1,
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

    pub(super) fn table_mut(&mut self) -> &mut [VariablePositionInBody] {
        &mut self.table
    }

    pub fn get_entry_for_variable(&self, variable_index: usize) -> Option<&VariablePositionInBody> {
        self.variable_index_to_table_index
            .get(&variable_index)
            .map(|&table_index| &self.table[table_index])
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

    pub fn add_indirect_entry(
        &mut self,
        variable_index: usize,
        condition_index: usize,
        indirect_table_index: usize,
    ) {
        let table_index = self
            .variable_index_to_table_index
            .get(&variable_index)
            .copied();
        match table_index {
            Some(table_index) => {
                panic!(
                    "Variable {:?} already has an entry in the table at index {}",
                    variable_index, table_index
                );
            }
            None => {
                let our_table_index = self.table.len();
                self.table.push(VariablePositionInBody::Indirect {
                    condition_index,
                    table_index: indirect_table_index,
                });
                self.variable_index_to_table_index
                    .insert(variable_index, our_table_index);
                self.table_index_to_variable_index
                    .insert(our_table_index, variable_index);
            }
        }
    }
}

impl Display for VariableSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VariableSource {{")?;
        for (i, position) in self.table.iter().enumerate() {
            write!(
                f,
                "\n  ?{}: {}",
                self.table_index_to_variable_index[&i], position
            )?;
        }
        write!(f, "\n}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::datalog::{arguments::Arguments, term::Term};

    #[test]
    fn test_variable_position_map() {
        let effect = Atom::new(
            Arguments::new(vec![Term::new_object(0), Term::new_variable(3)]),
            0,
            false,
        );

        let variable_position_map = VariablePositionInEffect::new(&effect);
        assert_eq!(variable_position_map.get(0), None);
        assert_eq!(variable_position_map.get(3), Some(1));
    }

    // TODO-soon: Add tests for VariableSource
}
