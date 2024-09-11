use crate::search::database::{utils::compute_matching_columns, Table};

use std::collections::{HashMap, HashSet};

/// Same as [`super::semi_join::semi_join`], but uses a hash map to speed up the
/// process.
#[allow(dead_code)]
pub fn hash_semi_join(t1: &mut Table, t2: &Table) -> usize {
    let matches = compute_matching_columns(t1, t2);

    if matches.is_empty() {
        return t1.tuples.len();
    }

    // TODO-someday we don't actually need a [`HashMap`] here, a [`HashSet`]
    // would be enough
    let mut hash_join_map = HashMap::new();
    for tuple_t2 in &t2.tuples {
        let key = matches
            .iter()
            .map(|(_, i)| tuple_t2[*i])
            .collect::<Vec<_>>();
        hash_join_map
            .entry(key)
            .or_insert_with(HashSet::new)
            .insert(tuple_t2);
    }

    t1.tuples.retain(|tuple_t1| {
        let key = matches
            .iter()
            .map(|(i, _)| tuple_t1[*i])
            .collect::<Vec<_>>();

        hash_join_map.contains_key(&key)
    });

    t1.tuples.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{small_tuple, SmallTuple};

    #[test]
    fn test_semi_join() {
        let mut t1 = Table::new(
            vec![
                small_tuple![1, 2, 3],
                small_tuple![1, 2, 4],
                small_tuple![3, 2, 3],
                small_tuple![3, 5, 1],
            ],
            vec![0, 1, 2],
        );
        let t2 = Table::new(
            vec![small_tuple![2, 3, 5], small_tuple![5, 1, 2]],
            vec![1, 2, 3],
        );

        assert_eq!(hash_semi_join(&mut t1, &t2), 3);
        let expected_tuples: Vec<SmallTuple> = vec![
            small_tuple![1, 2, 3],
            small_tuple![3, 2, 3],
            small_tuple![3, 5, 1],
        ];
        assert_eq!(t1.tuples, expected_tuples);
        assert_eq!(t1.tuple_index, vec![0, 1, 2]);
    }

    #[test]
    fn test_semi_join_no_match() {
        let mut t1 = Table::new(vec![small_tuple![1, 2], small_tuple![1, 4]], vec![0, 1]);
        let t2 = Table::new(vec![small_tuple![2, 3]], vec![2, 3]);

        assert_eq!(hash_semi_join(&mut t1, &t2), 2);
        let expected_tupels: Vec<SmallTuple> = vec![small_tuple![1, 2], small_tuple![1, 4]];
        assert_eq!(t1.tuples, expected_tupels);
        assert_eq!(t1.tuple_index, vec![0, 1]);
    }
}
