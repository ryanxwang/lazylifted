use crate::parsed_types::{Atom, Literal, Name, NameLiteral};
use crate::search::states::DBState;
use std::collections::HashMap;

/// A single goal atom. The arguments are indices into the task's object list.
#[derive(Debug)]
pub struct GoalAtom {
    pub predicate_index: usize,
    pub arguments: Vec<usize>,
    pub negated: bool,
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
    pub atoms: Vec<GoalAtom>,
    pub positive_nullary_goals: Vec<usize>,
    pub negative_nullary_goals: Vec<usize>,
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

    /// Returns true if the goal is satisfied by the given state.
    pub fn is_satisfied(&self, state: &DBState) -> bool {
        for &pred in &self.positive_nullary_goals {
            if !state.nullary_atoms[pred] {
                return false;
            }
        }

        for &pred in &self.negative_nullary_goals {
            if state.nullary_atoms[pred] {
                return false;
            }
        }

        for atom in &self.atoms {
            let goal_predicate = atom.predicate_index;
            let relations = &state.relations[goal_predicate];
            debug_assert!(relations.predicate_symbol == goal_predicate);

            if relations.tuples.contains(&atom.arguments) == atom.negated {
                return false;
            }
        }

        true
    }
}
