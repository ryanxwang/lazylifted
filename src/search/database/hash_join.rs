use std::collections::{HashMap, HashSet};

use crate::search::database::{utils::compute_matching_columns, Table};

/// Same as [`super::join::join`], but uses a hash map to speed up the process.
pub fn hash_join(t1: &mut Table, t2: &Table) {
    let matches = compute_matching_columns(t1, t2);

    if matches.is_empty() {
        // If there are no matching columns, we apply a cartesian product
        t1.tuple_index.extend(t2.tuple_index.iter().cloned());
        let mut cartesian_product = Vec::new();
        for tuple1 in &t1.tuples {
            for tuple2 in &t2.tuples {
                let mut new_tuple = tuple1.clone();
                new_tuple.extend(tuple2.iter().cloned());
                cartesian_product.push(new_tuple);
            }
        }
        t1.tuples = cartesian_product;
    } else {
        let mut hash_join_map = HashMap::new();

        for tuple_t2 in &t2.tuples {
            let key = matches
                .iter()
                .map(|(_, i)| tuple_t2[*i])
                .collect::<Vec<_>>();
            hash_join_map
                .entry(key)
                .or_insert_with(Vec::new)
                .push(tuple_t2);
        }

        let to_remove = matches.iter().map(|(_, j)| *j).collect::<HashSet<_>>();
        t1.tuple_index.extend(
            t2.tuple_index
                .iter()
                .enumerate()
                .filter(|(j, _)| !to_remove.contains(j))
                .map(|(_, c)| *c),
        );

        t1.tuples = t1
            .tuples
            .iter()
            .flat_map(|tuple1| {
                let key = matches.iter().map(|(i, _)| tuple1[*i]).collect::<Vec<_>>();

                hash_join_map.get(&key).into_iter().flat_map(|tuples| {
                    tuples.iter().map(|tuple2| {
                        let mut new_tuple = tuple1.clone();
                        new_tuple.extend(
                            tuple2
                                .iter()
                                .enumerate()
                                .filter(|(j, _)| !to_remove.contains(j))
                                .map(|(_, c)| *c),
                        );
                        new_tuple
                    })
                })
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        let mut t1 = Table::new(
            vec![vec![1, 2, 3], vec![1, 2, 4], vec![3, 2, 3], vec![3, 5, 1]],
            vec![0, 1, 2],
        );
        let t2 = Table::new(
            vec![vec![2, 3, 5], vec![2, 3, 7], vec![5, 1, 2]],
            vec![1, 2, 3],
        );

        hash_join(&mut t1, &t2);

        assert_eq!(
            t1.tuples,
            vec![
                vec![1, 2, 3, 5],
                vec![1, 2, 3, 7],
                vec![3, 2, 3, 5],
                vec![3, 2, 3, 7],
                vec![3, 5, 1, 2]
            ]
        );
        assert_eq!(t1.tuple_index, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_join_no_match() {
        let mut t1 = Table::new(vec![vec![1, 2], vec![1, 4]], vec![0, 1]);
        let t2 = Table::new(vec![vec![2, 3], vec![2, 5]], vec![2, 3]);

        hash_join(&mut t1, &t2);

        assert_eq!(
            t1.tuples,
            vec![
                vec![1, 2, 2, 3],
                vec![1, 2, 2, 5],
                vec![1, 4, 2, 3],
                vec![1, 4, 2, 5]
            ]
        );
        assert_eq!(t1.tuple_index, vec![0, 1, 2, 3]);
    }
}
