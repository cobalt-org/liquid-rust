use std::fmt;
use std::slice;

use itertools;

use super::Scalar;

/// Path to a value in an `Object`.
///
/// There is guaranteed always at least one element.
#[derive(Clone, Debug, PartialEq)]
pub struct Path(Vec<Scalar>);

impl Path {
    /// Create a `Value` reference.
    pub fn with_index<I: Into<Scalar>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Path(indexes)
    }

    /// Append an index.
    pub fn push<I: Into<Scalar>>(&mut self, value: I) {
        self.0.push(value.into());
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `Path`. The `Path` may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// Access the `Value` reference.
    pub fn iter(&self) -> PathIter {
        PathIter(self.0.iter())
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[Scalar] {
        self.0.as_slice()
    }
}

impl Extend<Scalar> for Path {
    fn extend<T: IntoIterator<Item = Scalar>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl ::std::ops::Deref for Path {
    type Target = [Scalar];

    #[inline]
    fn deref( &self ) -> &Self::Target {
        &self.0
    }
}

impl ::std::borrow::Borrow<[Scalar]> for Path {
    #[inline]
    fn borrow(&self) -> &[Scalar] {
        self
    }
}

impl AsRef<[Scalar]> for Path {
    #[inline]
    fn as_ref(&self) -> &[Scalar] {
        self
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
pub struct PathIter<'i>(slice::Iter<'i, Scalar>);

impl<'i> Iterator for PathIter<'i> {
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

impl<'i> ExactSizeIterator for PathIter<'i>{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Path to a value in an `Object`.
pub type PathRef<'s> = &'s [Scalar];
