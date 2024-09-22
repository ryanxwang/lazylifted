use std::{ops::Index, slice::SliceIndex};

use crate::search::datalog::term::Term;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Arguments {
    terms: Vec<Term>,
}

impl Arguments {
    pub fn new(terms: Vec<Term>) -> Self {
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

impl IntoIterator for Arguments {
    type Item = Term;
    type IntoIter = std::vec::IntoIter<Self::Item>;

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
