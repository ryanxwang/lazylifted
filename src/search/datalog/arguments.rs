use smallvec::SmallVec;
use std::{
    fmt::{Display, Formatter},
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::search::{datalog::term::Term, TYPICAL_NUM_ARGUMENTS};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Arguments {
    terms: SmallVec<[Term; TYPICAL_NUM_ARGUMENTS]>,
}

impl Arguments {
    pub fn new(terms: SmallVec<[Term; TYPICAL_NUM_ARGUMENTS]>) -> Self {
        Self { terms }
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Term> {
        self.terms.iter()
    }
}

impl<I: SliceIndex<[Term]>> Index<I> for Arguments {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.terms[index]
    }
}

impl<I: SliceIndex<[Term]>> IndexMut<I> for Arguments {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.terms[index]
    }
}

impl IntoIterator for Arguments {
    type Item = Term;
    type IntoIter = smallvec::IntoIter<[Term; 5]>;

    fn into_iter(self) -> Self::IntoIter {
        self.terms.into_iter()
    }
}

impl<'a> IntoIterator for &'a Arguments {
    type Item = &'a Term;
    type IntoIter = std::slice::Iter<'a, Term>;

    fn into_iter(self) -> Self::IntoIter {
        self.terms.iter()
    }
}

impl<'a> IntoIterator for &'a mut Arguments {
    type Item = &'a mut Term;
    type IntoIter = std::slice::IterMut<'a, Term>;

    fn into_iter(self) -> Self::IntoIter {
        self.terms.iter_mut()
    }
}

impl Display for Arguments {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, term) in self.terms.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", term)?;
        }
        write!(f, ")")
    }
}
