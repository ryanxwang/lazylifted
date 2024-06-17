use crate::search::database::{utils::compute_matching_columns, Table};

/// Semi-join two tables, modifying the first table in place to be the result of
/// the semi-join. Returns the size of the result. Note that nothing is done if
/// the two tables have no matching columns.
///
/// See also [`super::hash_semi_join::hash_semi_join`].
pub fn semi_join(t1: &mut Table, t2: &Table) -> usize {
    let matches = compute_matching_columns(t1, t2);

    if matches.is_empty() {
        return t1.tuples.len();
    }

    t1.tuples.retain(|tuple_t1| {
        for tuple_t2 in &t2.tuples {
            let mut match_found = true;

            for (i, j) in &matches {
                if tuple_t1[*i] != tuple_t2[*j] {
                    match_found = false;
                    break;
                }
            }

            if match_found {
                return true;
            }
        }

        false
    });

    t1.tuples.len()
}

#[cfg(test)]
mod tests {
    use crate::search::database::table::Tuple;

    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_semi_join() {
        let mut t1 = Table::new(
            vec![
                smallvec![1, 2, 3],
                smallvec![1, 2, 4],
                smallvec![3, 2, 3],
                smallvec![3, 5, 1],
            ],
            vec![0, 1, 2],
        );
        let t2 = Table::new(vec![smallvec![2, 3, 5], smallvec![5, 1, 2]], vec![1, 2, 3]);

        assert_eq!(semi_join(&mut t1, &t2), 3);
        let expected_tuples: Vec<Tuple> =
            vec![smallvec![1, 2, 3], smallvec![3, 2, 3], smallvec![3, 5, 1]];
        assert_eq!(t1.tuples, expected_tuples);
        assert_eq!(t1.tuple_index, vec![0, 1, 2]);
    }

    #[test]
    fn test_semi_join_no_match() {
        let mut t1 = Table::new(vec![smallvec![1, 2], smallvec![1, 4]], vec![0, 1]);
        let t2 = Table::new(vec![smallvec![2, 3]], vec![2, 3]);

        assert_eq!(semi_join(&mut t1, &t2), 2);
        let expected_tuples: Vec<Tuple> = vec![smallvec![1, 2], smallvec![1, 4]];
        assert_eq!(t1.tuples, expected_tuples);
        assert_eq!(t1.tuple_index, vec![0, 1]);
    }
}
