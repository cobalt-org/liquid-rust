use std::fmt;
use std::iter;
use std::slice;

use itertools;

use super::Scalar;

/// Path to a value in an `Object`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    indexes: Vec<Scalar>,
}

impl Path {
    /// Create a `Path` from iterator of `Scalar`s
    pub fn new<T: IntoIterator<Item = Scalar>>(indexes: T) -> Self {
        let indexes = indexes.into_iter().collect();
        Self { indexes }
    }

    /// Create a `Value` reference.
    pub fn with_index<I: Into<Scalar>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Self { indexes }
    }

    /// Append an index.
    pub fn push<I: Into<Scalar>>(mut self, value: I) -> Self {
        self.indexes.push(value.into());
        self
    }

    /// Removes the last index from the path and returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<Scalar> {
        self.indexes.pop()
    }

    /// Access the `Value` reference.
    pub fn iter(&self) -> ScalarIter {
        ScalarIter(self.indexes.iter())
    }
}

impl Extend<Scalar> for Path {
    fn extend<T: IntoIterator<Item = Scalar>>(&mut self, iter: T) {
        self.indexes.extend(iter);
    }
}

impl iter::FromIterator<Scalar> for Path {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Scalar>,
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
pub struct ScalarIter<'i>(slice::Iter<'i, Scalar>);

impl<'i> Iterator for ScalarIter<'i> {
    type Item = &'i Scalar;

    #[inline]
    fn next(&mut self) -> Option<&'i Scalar> {
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
