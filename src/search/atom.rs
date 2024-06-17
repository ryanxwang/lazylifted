use crate::parsed_types::{Atom as ParsedAtom, Name};
use crate::search::{object_tuple, AtomSchema, Negatable, ObjectTuple, SchemaArgument, Task};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An [`Atom`] is similar to a [`crate::search::AtomSchema`], except that
/// state atoms are fully grounded.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Atom {
    predicate_index: usize,
    arguments: ObjectTuple,
}

impl Atom {
    pub fn new(predicate_index: usize, arguments: ObjectTuple) -> Self {
        Self {
            predicate_index,
            arguments,
        }
    }

    pub fn from_parsed(
        atom: &ParsedAtom<Name>,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        let predicate_index = *predicate_table
            .get(atom.predicate_name())
            .expect("Goal atom predicate not found in domain predicate table.");
        let arguments = atom
            .values()
            .iter()
            .map(|name| {
                *object_table
                    .get(name)
                    .expect("Goal atom argument not found in object table.")
            })
            .collect();

        Self {
            predicate_index,
            arguments,
        }
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.predicate_index
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[usize] {
        &self.arguments
    }

    pub fn human_readable(&self, task: &Task) -> String {
        format!(
            "({} {})",
            task.predicates[self.predicate_index].name,
            self.arguments
                .iter()
                .map(|&arg| task.objects[arg].name.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl Negatable<Atom> {
    pub fn new_atom(
        atom: &ParsedAtom<Name>,
        negated: bool,
        predicate_table: &HashMap<Name, usize>,
        object_table: &HashMap<Name, usize>,
    ) -> Self {
        Negatable::new(
            negated,
            Atom::from_parsed(atom, predicate_table, object_table),
        )
    }

    #[inline(always)]
    pub fn predicate_index(&self) -> usize {
        self.underlying().predicate_index()
    }

    #[inline(always)]
    pub fn arguments(&self) -> &[usize] {
        self.underlying().arguments()
    }
}

impl TryFrom<AtomSchema> for Atom {
    type Error = ();

    fn try_from(value: AtomSchema) -> Result<Self, Self::Error> {
        let mut arguments = object_tuple![];
        for argument in value.arguments() {
            match argument {
                SchemaArgument::Constant(index) => arguments.push(*index),
                SchemaArgument::Free(_) => {
                    return Err(());
                }
            }
        }

        Ok(Self::new(value.predicate_index(), arguments))
    }
}

impl TryFrom<Negatable<AtomSchema>> for Negatable<Atom> {
    type Error = ();

    fn try_from(value: Negatable<AtomSchema>) -> Result<Self, Self::Error> {
        Ok(Negatable::new(
            value.is_negated(),
            Atom::try_from(value.underlying().to_owned())?,
        ))
    }
}
