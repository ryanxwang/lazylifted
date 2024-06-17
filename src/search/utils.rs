use smallvec::SmallVec;

const TYPICAL_NUM_ARGUMENTS: usize = 5;
pub type ObjectTuple = SmallVec<[usize; TYPICAL_NUM_ARGUMENTS]>;

// based on [`smallvec::smallvec`]
#[allow(unused_macros)]
macro_rules! object_tuple {
    // count helper: transform any expression into 1
    (@one $x:expr) => (1usize);
    ($elem:expr; $n:expr) => ({
        $crate::search::ObjectTuple::from_elem($elem, $n)
    });
    ($($x:expr),*$(,)*) => ({
        let count = 0usize $(+ $crate::search::object_tuple!(@one $x))*;
        #[allow(unused_mut)]
        let mut vec = $crate::search::ObjectTuple::new();
        if count <= vec.inline_size() {
            $(vec.push($x);)*
            vec
        } else {
            $crate::search::ObjectTuple::from_vec(smallvec::alloc::vec![$($x,)*])
        }
    });
}
pub(crate) use object_tuple;
