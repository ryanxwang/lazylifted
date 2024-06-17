use smallvec::SmallVec;

const TYPICAL_TUPLE_SIZE: usize = 5;
pub type Tuple = SmallVec<[usize; TYPICAL_TUPLE_SIZE]>;

/// Data structure containing a set of tuples and indices corresponding to the
/// free variable index in each tuple position.
#[derive(Debug, Clone)]
pub struct Table {
    pub tuples: Vec<Tuple>,
    pub tuple_index: Vec<i32>,
}

impl Table {
    pub fn new(tuples: Vec<Tuple>, tuple_index: Vec<i32>) -> Self {
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
