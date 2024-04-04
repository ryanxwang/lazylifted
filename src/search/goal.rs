use std::collections::HashMap;

use crate::parsed_types::{Atom, Literal, Name, NameLiteral};

/// A single goal atom. The arguments are indices into the task's object list.
#[derive(Debug)]
pub struct GoalAtom {
    predicate_index: usize,
    arguments: Vec<usize>,
    negated: bool,
}

impl GoalAtom {
    /// Creates a new goal atom.
    pub fn new(
        atom: &Atom<Name>,
        negated: bool,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        debug_assert!(!atom.values().is_empty());

        let predicate_index = predicate_table
            .get(atom.predicate_name())
            .expect("Goal atom predicate not found in domain predicate table.")
            .clone();
        let arguments = atom
            .values()
            .iter()
            .map(|name| {
                object_table
                    .get(name)
                    .expect("Goal atom argument not found in object table.")
                    .clone()
            })
            .collect();

        Self {
            predicate_index,
            arguments,
            negated,
        }
    }
}

/// The goal of a task.
#[derive(Debug)]
pub struct Goal {
    atoms: Vec<GoalAtom>,
    positive_nullary_goals: Vec<usize>,
    negative_nullary_goals: Vec<usize>,
}

impl Goal {
    /// Creates a new goal.
    pub fn new(
        goal: &Vec<NameLiteral>,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let mut atoms = vec![];
        let mut positive_nullary_goals = vec![];
        let mut negative_nullary_goals = vec![];

        for literal in goal {
            let (atom, negated) = match literal {
                Literal::Positive(atom) => (atom, false),
                Literal::Negative(atom) => (atom, true),
            };

            if atom.values().is_empty() {
                let pred_index = predicate_table
                    .get(atom.predicate_name())
                    .expect("Goal predicate not found in domain predicate table.")
                    .clone();
                if negated {
                    negative_nullary_goals.push(pred_index);
                } else {
                    positive_nullary_goals.push(pred_index);
                }
            } else {
                atoms.push(GoalAtom::new(
                    atom,
                    negated,
                    &predicate_table,
                    &object_table,
                ));
            }
        }

        Self {
            atoms,
            positive_nullary_goals,
            negative_nullary_goals,
        }
    }
}
