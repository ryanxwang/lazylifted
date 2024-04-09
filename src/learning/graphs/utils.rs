use crate::search::{DBState, Goal, SchemaArgument};

/// A simple representation of an atom in a goal. The first element is the
/// predicate index, the second element is a list of object indices.
pub type Atom = (usize, Vec<usize>);

/// A representation of a schema predication, which is basically just a
/// [`crate::search::SchemaAtom`] without the
/// [`crate::search::SchemaAtom::negated`] field.
pub type SchemaPred = (usize, Vec<SchemaArgument>);

/// Returns a list of all atoms in the goal, including nullary atoms.
/// Nullary atoms are represented as atoms with no arguments. This function
/// panics for any negative goal atom, since they are not supported by any of
/// the graph compilers.
pub fn atoms_of_goal(goal: &Goal) -> Vec<Atom> {
    let mut atoms: Vec<Atom> = goal
        .atoms
        .iter()
        .map(|atom| {
            assert!(!atom.negated);
            (atom.predicate_index, atom.arguments.clone())
        })
        .collect();

    for &pred in &goal.positive_nullary_goals {
        atoms.push((pred, vec![]));
    }
    assert!(goal.negative_nullary_goals.is_empty());
    atoms
}

pub fn atoms_of_state(state: &DBState) -> Vec<Atom> {
    let mut atoms = vec![];

    for relation in &state.relations {
        let pred = relation.predicate_symbol;
        for tuple in &relation.tuples {
            atoms.push((pred, tuple.clone()));
        }
    }
    for (i, &nullary) in state.nullary_atoms.iter().enumerate() {
        if nullary {
            atoms.push((i, vec![]));
        }
    }

    atoms
}
