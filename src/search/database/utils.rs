use crate::search::database::Table;

pub fn compute_matching_columns(t1: &Table, t2: &Table) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    for (i, c1) in t1.tuple_index.iter().enumerate() {
        for (j, c2) in t2.tuple_index.iter().enumerate() {
            if c1 == c2 {
                matches.push((i, j));
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use crate::search::small_tuple;

    use super::*;

    #[test]
    fn test_compute_matching_columns() {
        let t1 = Table::new(
            vec![small_tuple![1, 2, 3], small_tuple![1, 2, 4]],
            vec![1, 2],
        );
        let t2 = Table::new(
            vec![small_tuple![1, 2, 3], small_tuple![1, 2, 4]],
            vec![0, 1],
        );

        assert_eq!(compute_matching_columns(&t1, &t2), vec![(0, 1)]);
    }
}
