use crate::parsed_types::{Literal, Name, NameLiteral};
use crate::search::{Atom, DBState, Negatable};
use std::collections::HashMap;

/// The goal of a task.
#[derive(Debug)]
pub struct Goal {
    atoms: Vec<Negatable<Atom>>,
}

impl Goal {
    /// Creates a new goal.
    pub fn new(
        goal: &[NameLiteral],
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let mut atoms = vec![];

        for literal in goal {
            let (atom, negated) = match literal {
                Literal::Positive(atom) => (atom, false),
                Literal::Negative(atom) => (atom, true),
            };

            atoms.push(Negatable::new_atom(
                atom,
                negated,
                predicate_table,
                object_table,
            ));
        }

        Self { atoms }
    }

    /// Returns true if the goal is satisfied by the given state.
    pub fn is_satisfied(&self, state: &DBState) -> bool {
        for atom in &self.atoms {
            if !state.satisfied(atom) {
                return false;
            }
        }

        true
    }

    pub fn atoms(&self) -> &Vec<Negatable<Atom>> {
        &self.atoms
    }
}
