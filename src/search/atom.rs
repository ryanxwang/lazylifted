use crate::parsed_types::{Atom as ParsedAtom, Name};
use crate::search::{AtomSchema, Negatable, SchemaArgument, SmallTuple, Task};
use std::collections::HashMap;

use super::raw_small_tuple;

/// An [`Atom`] is similar to a [`crate::search::AtomSchema`], except that
/// state atoms are fully grounded.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Atom {
    predicate_index: usize,
    arguments: SmallTuple,
}

impl Atom {
    pub fn new(predicate_index: usize, arguments: SmallTuple) -> Self {
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
        let arguments = SmallTuple::new(
            atom.values()
                .iter()
                .map(|name| {
                    *object_table
                        .get(name)
                        .expect("Goal atom argument not found in object table.")
                })
                .collect(),
        );

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
    pub fn arguments(&self) -> &SmallTuple {
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
    pub fn arguments(&self) -> &SmallTuple {
        self.underlying().arguments()
    }
}

impl TryFrom<AtomSchema> for Atom {
    type Error = ();

    fn try_from(value: AtomSchema) -> Result<Self, Self::Error> {
        let mut arguments = raw_small_tuple![];
        for argument in value.arguments() {
            match argument {
                SchemaArgument::Constant(index) => arguments.push(*index),
                SchemaArgument::Free {
                    variable_index: _,
                    type_index: _,
                } => {
                    return Err(());
                }
            }
        }

        Ok(Self::new(value.predicate_index(), arguments.into()))
    }
}

impl TryFrom<Negatable<AtomSchema>> for Negatable<Atom> {
    type Error = ();

    fn try_from(value: Negatable<AtomSchema>) -> Result<Self, Self::Error> {
        Ok(Negatable::new(
            value.is_negative(),
            Atom::try_from(value.underlying().to_owned())?,
        ))
    }
}
