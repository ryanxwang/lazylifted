use crate::search::{
    raw_small_tuple,
    states::{DBState, Relation},
    RawSmallTuple, SmallTuple, Task,
};
use internment::Intern;
use lru::LruCache;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    num::NonZeroUsize,
};

/// The [`InternalSparsePackedState`] struct is used to store a state in a more
/// compact representation. This is based on the powerlifted implementation,
/// which is then based on the fast downward implementation.
///
/// We represent a state as a vector of relations and a vector of nullary atoms.
/// Each relation is a set of tuples, which can be interpreted as a 'table'. In
/// order to make the representation more concise, we first order the tuples in
/// the sets corresponding to each relation in a deterministic way. We then hash
/// these sets in a well-defined order (by the predicate symbol of the
/// corresponding relation). Last, we combine all these hash values together and
/// also combine with a hash over the predicate symbols and the truth value of
/// the nullary relations (i.e., predicates) of the state.
///
/// This type is only made "pub" to satisfy the compiler. It should not be used
/// outside of this module. Use its interned version, [`SparsePackedState`],
/// instead.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct InternalSparsePackedState {
    packed_relations: Vec<Vec<u64>>,
    predicate_symbols: Vec<usize>,
    nullary_atoms: Vec<bool>,
}

/// The [`SparsePackedState`] type is an interned version of the internal
/// representation of a state. This is useful for avoiding having many copies of
/// the same state in memory, which can happen to a scary degree for partial
/// space search.
pub type SparsePackedState = Intern<InternalSparsePackedState>;

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
    // We cache only for fast unpacking but not for packing, as we never pack
    // the same state twice.
    unpacked_states_cache: RefCell<LruCache<SparsePackedState, DBState>>,
    static_predicate_indices: HashSet<usize>,
    static_relations: HashMap<usize, Relation>,
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

        // We deal with static predicates by not packing them at all. Instead,
        // we store them in the packer and return them as is when unpacking.
        let static_predicate_indices = task.static_predicates().clone();
        let mut static_relations = HashMap::new();
        for &i in &static_predicate_indices {
            static_relations.insert(i, task.initial_state.relations[i].clone());
        }

        Self {
            hash_multipliers,
            obj_to_hash_index,
            hash_index_to_obj,
            unpacked_states_cache: RefCell::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            static_predicate_indices,
            static_relations,
        }
    }

    pub fn pack(&self, state: &DBState) -> SparsePackedState {
        let mut packed_relations = Vec::with_capacity(state.relations.len());
        let mut predicate_symbols = Vec::with_capacity(state.relations.len());

        for relation in &state.relations {
            let predicate_index = relation.predicate_symbol;

            // If the predicate is static, we don't need to pack it, it will be the
            // same in all states.
            if self.static_predicate_indices.contains(&predicate_index) {
                packed_relations.push(Vec::new());
                predicate_symbols.push(predicate_index);
                continue;
            }

            let mut packed_relation = Vec::with_capacity(relation.tuples.len());
            for tuple in relation.tuples.iter() {
                let mut hash = 0;
                for (i, &x) in tuple.iter().enumerate() {
                    hash += self.hash_multipliers[predicate_index][i]
                        * self.obj_to_hash_index[predicate_index][i][&x] as u64;
                }
                packed_relation.push(hash);
            }
            // sorting ensures that the hash is unique for each state
            packed_relation.sort_unstable();
            packed_relations.push(packed_relation);
            predicate_symbols.push(predicate_index);
        }

        // rywang: I've thought about adding to the unpacking cache here as
        // well. That turns out to significantly increase the time spent
        // packing, mainly due to having to clone, and doesn't help much - the
        // situation could change though.

        Intern::new(InternalSparsePackedState {
            packed_relations,
            predicate_symbols,
            nullary_atoms: state.nullary_atoms.clone(),
        })
    }

    pub fn unpack(&self, packed_state: &SparsePackedState) -> DBState {
        if let Some(state) = self.unpacked_states_cache.borrow_mut().get(packed_state) {
            return state.clone();
        }

        let mut relations = Vec::with_capacity(packed_state.packed_relations.len());

        for (i, packed_relation) in packed_state.packed_relations.iter().enumerate() {
            if self
                .static_predicate_indices
                .contains(&packed_state.predicate_symbols[i])
            {
                relations.push(self.static_relations[&packed_state.predicate_symbols[i]].clone());
                continue;
            }

            let mut tuples = BTreeSet::new();
            let predicate_index = packed_state.predicate_symbols[i];
            for &hash in packed_relation.iter() {
                let mut tuple: RawSmallTuple =
                    raw_small_tuple![0; self.hash_multipliers[predicate_index].len()];
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
                tuples.insert(SmallTuple::new(tuple));
            }
            relations.push(Relation {
                predicate_symbol: predicate_index,
                tuples,
            })
        }

        let dbstate = DBState {
            relations,
            nullary_atoms: packed_state.nullary_atoms.clone(),
        };

        self.unpacked_states_cache
            .borrow_mut()
            .put(packed_state.to_owned(), dbstate.clone());

        dbstate
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
