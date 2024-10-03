use std::{
    fmt::{Debug, Formatter},
    ops::Index,
};

use internment::Intern;
use smallvec::SmallVec;

pub const TYPICAL_NUM_ARGUMENTS: usize = 5;

/// A [`RawSmallTuple`] is a small vector of `usize` that is used to represent a
/// tuple. Use this type while you still need to manipulate the tuple,
/// afterwards convert it to a [`SmallTuple`].
pub type RawSmallTuple = SmallVec<[usize; TYPICAL_NUM_ARGUMENTS]>;

/// A [`SmallTuple`] is a small vector of `usize` that is used to represent a
/// tuple. It is interned, so it is more efficient to store and compare, but it
/// can't be easily modified.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SmallTuple {
    inner: Intern<RawSmallTuple>,
}

impl SmallTuple {
    pub fn new(inner: RawSmallTuple) -> Self {
        Self {
            inner: Intern::new(inner),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn raw(&self) -> &RawSmallTuple {
        &self.inner
    }
}

impl From<RawSmallTuple> for SmallTuple {
    fn from(inner: RawSmallTuple) -> Self {
        Self::new(inner)
    }
}

impl From<&[usize]> for SmallTuple {
    fn from(inner: &[usize]) -> Self {
        Self::new(inner.into())
    }
}

impl From<Vec<usize>> for SmallTuple {
    fn from(inner: Vec<usize>) -> Self {
        Self::new(inner.into())
    }
}

impl Index<usize> for SmallTuple {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

// This custom implementation hides the internment details from the user.
impl Debug for SmallTuple {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl PartialEq<SmallTuple> for &SmallTuple {
    fn eq(&self, other: &SmallTuple) -> bool {
        self.inner == other.inner
    }
}

// based on [`smallvec::smallvec`]
macro_rules! small_tuple {
    // count helper: transform any expression into 1
    (@one $x:expr) => (1usize);
    ($elem:expr; $n:expr) => ({
        $crate::search::SmallTuple::new($crate::search::RawSmallTuple::from_elem($elem, $n))
    });
    ($($x:expr),*$(,)*) => ({
        let count = 0usize $(+ $crate::search::small_tuple!(@one $x))*;
        #[allow(unused_mut)]
        let mut vec = $crate::search::RawSmallTuple::new();
        if count <= vec.inline_size() {
            $(vec.push($x);)*
            $crate::search::SmallTuple::new(vec)
        } else {
            $crate::search::SmallTuple::new($crate::search::RawSmallTuple::from_vec(smallvec::alloc::vec![$($x,)*]))
        }
    });
}
pub(crate) use small_tuple;

macro_rules! raw_small_tuple {
    // count helper: transform any expression into 1
    (@one $x:expr) => (1usize);
    ($elem:expr; $n:expr) => ({
        $crate::search::RawSmallTuple::from_elem($elem, $n)
    });
    ($($x:expr),*$(,)*) => ({
        let count = 0usize $(+ $crate::search::raw_small_tuple!(@one $x))*;
        #[allow(unused_mut)]
        let mut vec = $crate::search::RawSmallTuple::new();
        if count <= vec.inline_size() {
            $(vec.push($x);)*
            vec
        } else {
            $crate::search::RawSmallTuple::from_vec(smallvec::alloc::vec![$($x,)*])
        }
    });
}
pub(crate) use raw_small_tuple;
