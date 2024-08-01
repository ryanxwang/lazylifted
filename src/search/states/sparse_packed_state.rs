use crate::search::{
    object_tuple,
    states::{DBState, GroundAtom, Relation},
    Task,
};
use std::collections::{BTreeSet, HashMap};

/// The [`SparsePackedState`] struct is used to store a state in a more compact
/// representation. This is based on the powerlifted implementation, which is
/// then based on the fast downward implementation.
///
/// We represent a state as a vector of relations and a vector of nullary atoms.
/// Each relation is a set of tuples, which can be interpreted as a 'table'. In
/// order to make the representation more concise, we first order the tuples in
/// the sets corresponding to each relation in a deterministic way. We then hash
/// these sets in a well-defined order (by the predicate symbol of the
/// corresponding relation). Last, we combine all these hash values together and
/// also combine with a hash over the predicate symbols and the truth value of
/// the nullary relations (i.e., predicates) of the state.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SparsePackedState {
    packed_relations: Vec<Vec<u64>>,
    predicate_symbols: Vec<usize>,
    nullary_atoms: Vec<bool>,
}

impl SparsePackedState {}

/// The [`SparseStatePacker`] struct is used to pack and unpack states into a
/// more compact representation. This is useful for storing states in a hash
/// table.
///
/// Its internal data structures have the following structure:
/// - First dimension: predicate index.
/// - Second dimension: type index (for each one of the predicate arguments).
///
/// The remaining dimensions are self-explanatory.
#[derive(Debug)]
pub struct SparseStatePacker {
    hash_multipliers: Vec<Vec<u64>>,
    obj_to_hash_index: Vec<Vec<HashMap<usize, usize>>>,
    hash_index_to_obj: Vec<Vec<HashMap<usize, usize>>>,
}

impl SparseStatePacker {
    pub fn new(task: &Task) -> Self {
        let mut hash_multipliers = Vec::with_capacity(task.predicates.len());
        let mut obj_to_hash_index = Vec::with_capacity(task.predicates.len());
        let mut hash_index_to_obj = Vec::with_capacity(task.predicates.len());
        let objects_per_type = task.objects_per_type();

        for i in 0..task.predicates.len() {
            let pred = &task.predicates[i];

            hash_multipliers.push(Vec::with_capacity(pred.types.len()));
            obj_to_hash_index.push(Vec::with_capacity(pred.types.len()));
            hash_index_to_obj.push(Vec::with_capacity(pred.types.len()));

            let mut multiplier: u64 = 1;
            for (j, &t) in pred.types.iter().enumerate() {
                hash_multipliers[i].push(multiplier);

                multiplier = multiplier
                    .checked_mul(objects_per_type[t].len() as u64)
                    .expect("Overflow in hash multiplier.");

                obj_to_hash_index[i].push(HashMap::new());
                hash_index_to_obj[i].push(HashMap::new());
                for (k, &obj) in objects_per_type[t].iter().enumerate() {
                    obj_to_hash_index[i][j].insert(obj, k);
                    hash_index_to_obj[i][j].insert(k, obj);
                }
            }
        }

        Self {
            hash_multipliers,
            obj_to_hash_index,
            hash_index_to_obj,
        }
    }

    pub fn pack(&self, state: &DBState) -> SparsePackedState {
        let mut packed_relations = Vec::with_capacity(state.relations.len());
        let mut predicate_symbols = Vec::with_capacity(state.relations.len());

        for relation in &state.relations {
            let mut packed_relation = Vec::with_capacity(relation.tuples.len());
            let predicate_index = relation.predicate_symbol;
            for tuple in relation.tuples.iter() {
                let mut hash = 0;
                for (i, &x) in tuple.iter().enumerate() {
                    hash += self.hash_multipliers[predicate_index][i]
                        * self.obj_to_hash_index[predicate_index][i][&x] as u64;
                }
                packed_relation.push(hash);
            }
            packed_relation.sort_unstable();
            packed_relations.push(packed_relation);
            predicate_symbols.push(relation.predicate_symbol);
        }

        SparsePackedState {
            packed_relations,
            predicate_symbols,
            nullary_atoms: state.nullary_atoms.clone(),
        }
    }

    pub fn unpack(&self, packed_state: &SparsePackedState) -> DBState {
        let mut relations = Vec::with_capacity(packed_state.packed_relations.len());

        for (i, packed_relation) in packed_state.packed_relations.iter().enumerate() {
            let mut tuples = BTreeSet::new();
            let predicate_index = packed_state.predicate_symbols[i];
            for &hash in packed_relation.iter() {
                let mut tuple: GroundAtom =
                    object_tuple![0; self.hash_multipliers[predicate_index].len()];
                let mut hash = hash;
                for j in (0..self.hash_multipliers[predicate_index].len()).rev() {
                    let multiplier = self.hash_multipliers[predicate_index][j];
                    let index = (hash / multiplier) as usize;
                    tuple[j] = *self.hash_index_to_obj[predicate_index][j]
                        .get(&index)
                        .unwrap();
                    hash -= multiplier * index as u64;
                }
                assert_eq!(hash, 0);
                tuples.insert(tuple);
            }
            relations.push(Relation {
                predicate_symbol: predicate_index,
                tuples,
            })
        }

        DBState {
            relations,
            nullary_atoms: packed_state.nullary_atoms.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn pack_then_unpack() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);

        let packer = SparseStatePacker::new(&task);
        let packed_state = packer.pack(&task.initial_state);
        let unpacked_state = packer.unpack(&packed_state);
        assert_eq!(task.initial_state, unpacked_state);
    }
}
