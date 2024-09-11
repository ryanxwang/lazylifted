use crate::search::SmallTuple;

/// Data structure containing a set of tuples and indices corresponding to the
/// free variable index in each tuple position.
#[derive(Debug, Clone)]
pub struct Table {
    pub tuples: Vec<SmallTuple>,
    pub tuple_index: Vec<i32>,
}

impl Table {
    pub fn new(tuples: Vec<SmallTuple>, tuple_index: Vec<i32>) -> Self {
        Self {
            tuples,
            tuple_index,
        }
    }

    pub fn index_is_variable(&self, index: usize) -> bool {
        self.tuple_index[index] >= 0
    }

    pub const EMPTY: Table = Table {
        tuples: Vec::new(),
        tuple_index: Vec::new(),
    };
}
