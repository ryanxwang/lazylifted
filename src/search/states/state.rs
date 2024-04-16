//! This module contains the definition of a search state. It is based on the
//! paper
//!
//! A. B. Correa, 2019.'Planning using Lifted Task Representations', M.Sc.
//! thesis. University of Basel.
//!
//! The implementation is based on that of powerlifted.

use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Display, Formatter},
};

use crate::parsed_types::{Literal, Name};

/// A ground atom is a vector of object indices. It only makes sense in the
/// context of a specific predicate, see [`Relation`].
pub type GroundAtom = Vec<usize>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Relation {
    /// The predicate symbol of this relation.
    pub predicate_symbol: usize,
    /// The tuples of the relation. This is a [`BTreeSet`] as [`HashSet`]
    /// does not implement [`Hash`] trait.
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
                    args.push(
                        *object_table.get(arg).unwrap_or_else(|| panic!("Object {} not found in object table {:?}",
                                arg, object_table)),
                    );
                }
                state.insert_tuple_in_relation(args, predicate_symbol);
            }
        }

        state
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
