use crate::search::database::Table;
use std::collections::HashSet;

/// Project the table over the given columns. This is not a canonical projection
/// as it does not remove the columns from the table's schema. This is so that
/// we can still instantiate the actions later on. When there are multiple rows
/// with the same values in the projected columns, only the first one is kept.
#[allow(dead_code)]
pub fn project(t: &mut Table, over: &HashSet<i32>) {
    let mut matches = Vec::new();
    for &x in over {
        for (i, &c) in t.tuple_index.iter().enumerate() {
            if c == x {
                matches.push(i);
            }
        }
    }

    let mut keys = HashSet::new();

    t.tuples.retain(|tuple| {
        let mut key = Vec::with_capacity(matches.len());
        for &i in &matches {
            key.push(tuple[i]);
        }

        keys.insert(key)
    });
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::search::{small_tuple, SmallTuple};

    #[test]
    fn test_project() {
        let mut table = Table::new(
            vec![
                small_tuple![1, 2, 3],
                small_tuple![1, 2, 4],
                small_tuple![1, 3, 4],
                small_tuple![2, 3, 4],
            ],
            vec![0, 1, 2],
        );

        let columns = HashSet::from_iter(vec![1, 2]);

        project(&mut table, &columns);

        assert_eq!(table.tuples.len(), 3);
        let expected_tuples: Vec<SmallTuple> = vec![
            small_tuple![1, 2, 3],
            small_tuple![1, 2, 4],
            small_tuple![1, 3, 4],
        ];
        assert_eq!(table.tuples, expected_tuples);
        assert_eq!(table.tuple_index, vec![0, 1, 2]);
    }
}
