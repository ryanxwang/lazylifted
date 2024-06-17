use std::collections::HashSet;

use crate::search::database::{utils::compute_matching_columns, Table};

/// Join two tables, modifying the first table in place to be the result of the
/// join. If there are no matching columns, a cartesian product is applied.
///
/// See also [`super::hash_join::hash_join`].
#[allow(dead_code)]
pub fn join(t1: &mut Table, t2: &Table) {
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
                t2.tuples.iter().filter_map(|tuple2| {
                    let mut match_found = true;

                    for (i, j) in &matches {
                        if tuple1[*i] != tuple2[*j] {
                            match_found = false;
                            break;
                        }
                    }

                    if match_found {
                        let mut new_tuple = tuple1.clone();
                        new_tuple.extend(
                            tuple2
                                .iter()
                                .enumerate()
                                .filter(|(j, _)| !to_remove.contains(j))
                                .map(|(_, c)| *c),
                        );
                        Some(new_tuple)
                    } else {
                        None
                    }
                })
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use crate::search::database::table::Tuple;

    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_join() {
        let mut t1 = Table::new(
            vec![
                smallvec![1, 2, 3],
                smallvec![1, 2, 4],
                smallvec![3, 2, 3],
                smallvec![3, 5, 1],
            ],
            vec![0, 1, 2],
        );
        let t2 = Table::new(
            vec![smallvec![2, 3, 5], smallvec![2, 3, 7], smallvec![5, 1, 2]],
            vec![1, 2, 3],
        );

        join(&mut t1, &t2);

        let expected_tuples: Vec<Tuple> = vec![
            smallvec![1, 2, 3, 5],
            smallvec![1, 2, 3, 7],
            smallvec![3, 2, 3, 5],
            smallvec![3, 2, 3, 7],
            smallvec![3, 5, 1, 2],
        ];
        assert_eq!(t1.tuples, expected_tuples);
        assert_eq!(t1.tuple_index, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_join_no_match() {
        let mut t1 = Table::new(vec![smallvec![1, 2], smallvec![1, 4]], vec![0, 1]);
        let t2 = Table::new(vec![smallvec![2, 3], smallvec![2, 5]], vec![2, 3]);

        join(&mut t1, &t2);

        let expected_tuples: Vec<Tuple> = vec![
            smallvec![1, 2, 2, 3],
            smallvec![1, 2, 2, 5],
            smallvec![1, 4, 2, 3],
            smallvec![1, 4, 2, 5],
        ];
        assert_eq!(t1.tuples, expected_tuples);
        assert_eq!(t1.tuple_index, vec![0, 1, 2, 3]);
    }
}
