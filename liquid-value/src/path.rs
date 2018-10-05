use std::fmt;
use std::iter;
use std::slice;

use itertools;

use super::Index;

/// Path to a value in an `Object`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    indexes: Vec<Index>,
}

impl Path {
    /// Create a `Path` from iterator of `Index`s
    pub fn new<T: IntoIterator<Item = Index>>(indexes: T) -> Self {
        let indexes = indexes.into_iter().collect();
        Self { indexes }
    }

    /// Create a `Value` reference.
    pub fn with_index<I: Into<Index>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Self { indexes }
    }

    /// Append an index.
    pub fn push<I: Into<Index>>(mut self, value: I) -> Self {
        self.indexes.push(value.into());
        self
    }

    /// Access the `Value` reference.
    pub fn iter(&self) -> IndexIter {
        IndexIter(self.indexes.iter())
    }
}

impl Extend<Index> for Path {
    fn extend<T: IntoIterator<Item = Index>>(&mut self, iter: T) {
        self.indexes.extend(iter);
    }
}

impl iter::FromIterator<Index> for Path {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Index>,
    {
        let indexes = iter.into_iter().collect();
        Self { indexes }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = itertools::join(self.iter(), ".");
        write!(f, "{}", data)
    }
}

/// Iterate over indexes in a `Value`'s `Path`.
#[derive(Debug)]
pub struct IndexIter<'i>(slice::Iter<'i, Index>);

impl<'i> Iterator for IndexIter<'i> {
    type Item = &'i Index;

    #[inline]
    fn next(&mut self) -> Option<&'i Index> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }
}
