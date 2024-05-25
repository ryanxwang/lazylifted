//! This module contains the definition of a search state. It is based on the
//! paper
//!
//! A. B. Correa, 2019.'Planning using Lifted Task Representations', M.Sc.
//! thesis. University of Basel.
//!
//! The implementation is based on that of powerlifted.

use crate::parsed_types::{Literal, Name};
use crate::search::{Atom, Negatable, Task};
use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Display, Formatter},
};

/// A ground atom is a vector of object indices. It only makes sense in the
/// context of a specific predicate, see [`Relation`].
pub type GroundAtom = Vec<usize>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Relation {
    /// The predicate symbol of this relation.
    pub predicate_symbol: usize,
    /// The tuples of the relation. This is a [`BTreeSet`] as
    /// [`HashSet`](std::collections::HashSet) does not implement [`Hash`] trait.
    pub tuples: BTreeSet<GroundAtom>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DBState {
    pub relations: Vec<Relation>,
    pub nullary_atoms: Vec<bool>,
}

impl DBState {
    pub fn new(num_predicates: usize) -> Self {
        DBState {
            relations: (0..num_predicates)
                .map(|i| Relation {
                    predicate_symbol: i,
                    tuples: BTreeSet::new(),
                })
                .collect(),
            nullary_atoms: vec![false; num_predicates],
        }
    }

    pub fn set_nullary_atom(&mut self, index: usize, v: bool) {
        self.nullary_atoms[index] = v;
    }

    pub fn insert_tuple_in_relation(&mut self, ga: GroundAtom, id: usize) {
        self.relations[id].tuples.insert(ga);
    }

    pub fn from_problem(
        problem: &crate::parsed_types::Problem,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let mut state = Self::new(predicate_table.len());

        for literal in problem.init() {
            let atom = match literal {
                Literal::Positive(atom) => atom,
                Literal::Negative(_) => {
                    panic!("Negative atoms in initial state do not make sense")
                }
            };

            let predicate_symbol = *predicate_table.get(atom.predicate_name()).unwrap();
            if atom.values().is_empty() {
                state.set_nullary_atom(predicate_symbol, true);
            } else {
                let mut args = Vec::with_capacity(atom.values().len());
                for arg in atom.values() {
                    args.push(*object_table.get(arg).unwrap_or_else(|| {
                        panic!(
                            "Object {} not found in object table {:?}",
                            arg, object_table
                        )
                    }));
                }
                state.insert_tuple_in_relation(args, predicate_symbol);
            }
        }

        state
    }

    pub fn satisfied(&self, atom: &Negatable<Atom>) -> bool {
        let goal_predicate = atom.predicate_index();
        let in_state = if atom.arguments().is_empty() {
            self.nullary_atoms[goal_predicate]
        } else {
            let relations = &self.relations[goal_predicate];
            assert_eq!(relations.predicate_symbol, goal_predicate);
            relations.tuples.contains(atom.arguments())
        };

        in_state != atom.is_negated()
    }

    pub fn atoms(&self) -> Vec<Atom> {
        let mut atoms = vec![];

        for relation in &self.relations {
            let pred = relation.predicate_symbol;
            for tuple in &relation.tuples {
                atoms.push(Atom::new(pred, tuple.clone()));
            }
        }

        for (i, &nullary) in self.nullary_atoms.iter().enumerate() {
            if nullary {
                atoms.push(Atom::new(i, vec![]));
            }
        }

        atoms
    }

    pub fn human_readable(&self, task: &Task) -> String {
        self.atoms()
            .into_iter()
            .map(|a| a.human_readable(task))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Display for DBState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, relation) in self.relations.iter().enumerate() {
            for tuple in &relation.tuples {
                write!(f, "({} {:?})", i, tuple)?;
            }
        }
        for (i, &nullary) in self.nullary_atoms.iter().enumerate() {
            if nullary {
                write!(f, "({})", i)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        search::Task,
        test_utils::{BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT},
    };

    use super::*;

    #[test]
    fn test_satisfied() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);

        let on_b1_b2 = Negatable::Positive(Atom::new(4, vec![0, 1]));
        let not_on_b1_b2 = Negatable::Negative(on_b1_b2.underlying().clone());
        assert!(task.initial_state.satisfied(&on_b1_b2));
        assert!(!task.initial_state.satisfied(&not_on_b1_b2));
    }
}
