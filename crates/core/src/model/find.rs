//! Find a `ValueView` nested in an `ObjectView`

use std::fmt;
use std::slice;

use crate::error::{Error, Result};
use crate::model::KStringCow;

use super::ScalarCow;
use super::Value;
use super::ValueCow;
use super::ValueView;

/// Path to a value in an `Object`.
///
/// There is guaranteed always at least one element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path<'s>(Vec<PathElement<'s>>);

/// Access kind for a variable path element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathAccess {
    /// Dot access like `foo.bar`.
    Dot,
    /// Bracket access like `foo["bar"]`.
    Lookup,
}

/// A single element in a variable path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathElement<'s> {
    value: ScalarCow<'s>,
    access: PathAccess,
}

impl<'s> PathElement<'s> {
    /// Create a path element with an explicit access kind.
    pub fn new<I: Into<ScalarCow<'s>>>(value: I, access: PathAccess) -> Self {
        Self {
            value: value.into(),
            access,
        }
    }

    /// Access the underlying scalar value.
    pub fn value(&self) -> &ScalarCow<'s> {
        &self.value
    }

    /// Returns true when the element came from bracket lookup syntax.
    pub fn is_lookup(&self) -> bool {
        matches!(self.access, PathAccess::Lookup)
    }
}

impl<'s, I: Into<ScalarCow<'s>>> From<I> for PathElement<'s> {
    fn from(value: I) -> Self {
        Self::new(value, PathAccess::Dot)
    }
}

impl<'s> Path<'s> {
    /// Create a `Value` reference.
    pub fn with_index<I: Into<PathElement<'s>>>(value: I) -> Self {
        let indexes = vec![value.into()];
        Path(indexes)
    }

    /// Append an index.
    pub fn push<I: Into<PathElement<'s>>>(&mut self, value: I) {
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
    pub fn iter(&self) -> PathIter<'_, '_> {
        PathIter(self.0.iter())
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[PathElement<'s>] {
        self.0.as_slice()
    }
}

impl<'s> Extend<PathElement<'s>> for Path<'s> {
    fn extend<T: IntoIterator<Item = PathElement<'s>>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<'s> ::std::ops::Deref for Path<'s> {
    type Target = [PathElement<'s>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> ::std::borrow::Borrow<[PathElement<'s>]> for Path<'s> {
    #[inline]
    fn borrow(&self) -> &[PathElement<'s>] {
        self
    }
}

impl<'s> AsRef<[PathElement<'s>]> for Path<'s> {
    #[inline]
    fn as_ref(&self) -> &[PathElement<'s>] {
        self
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = itertools::join(self.iter().map(|index| index.value().render()), ".");
        write!(f, "{}", data)
    }
}

/// Iterate over indexes in a `Value`'s `Path`.
#[derive(Debug)]
pub struct PathIter<'i, 's>(slice::Iter<'i, PathElement<'s>>);

impl<'i, 's: 'i> Iterator for PathIter<'i, 's> {
    type Item = &'i PathElement<'s>;

    #[inline]
    fn next(&mut self) -> Option<&'i PathElement<'s>> {
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

impl<'i, 's: 'i> ExactSizeIterator for PathIter<'i, 's> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Find a `ValueView` nested in an `ObjectView`
pub fn try_find<'o>(value: &'o dyn ValueView, path: &[PathElement<'_>]) -> Option<ValueCow<'o>> {
    let indexes = path.iter();
    try_find_borrowed(value, indexes)
}

fn try_find_borrowed<'o, 'i>(
    value: &'o dyn ValueView,
    mut path: impl Iterator<Item = &'i PathElement<'i>>,
) -> Option<ValueCow<'o>> {
    let index = match path.next() {
        Some(index) => index,
        None => {
            return Some(ValueCow::Borrowed(value));
        }
    };
    let child = augmented_get(value, index)?;
    match child {
        ValueCow::Owned(child) => try_find_owned(child, path),
        ValueCow::Borrowed(child) => try_find_borrowed(child, path),
        ValueCow::Shared(child) => {
            try_find_borrowed(child.as_ref(), path).map(|v| ValueCow::Owned(v.into_owned()))
        }
    }
}

fn try_find_owned<'o, 'i>(
    value: Value,
    mut path: impl Iterator<Item = &'i PathElement<'i>>,
) -> Option<ValueCow<'o>> {
    let index = match path.next() {
        Some(index) => index,
        None => {
            return Some(ValueCow::Owned(value));
        }
    };
    let child = augmented_get(&value, index)?;
    match child {
        ValueCow::Owned(child) => try_find_owned(child, path),
        ValueCow::Borrowed(child) => {
            try_find_borrowed(child, path).map(|v| ValueCow::Owned(v.into_owned()))
        }
        ValueCow::Shared(child) => {
            try_find_borrowed(child.as_ref(), path).map(|v| ValueCow::Owned(v.into_owned()))
        }
    }
}

fn augmented_get<'o>(value: &'o dyn ValueView, index: &PathElement<'_>) -> Option<ValueCow<'o>> {
    if let Some(arr) = value.as_array() {
        let lookup = index.is_lookup();
        let scalar = index.value();
        if !lookup || !scalar.is_string() {
            if let Some(index) = scalar.to_integer() {
                return arr.get(index).map(ValueCow::Borrowed);
            }
        }

        let index = scalar.to_kstr();
        if lookup {
            value
                .as_object()
                .and_then(|obj| obj.get(index.as_str()).map(ValueCow::Borrowed))
        } else {
            match index.as_str() {
                "first" => arr.first().map(ValueCow::Borrowed),
                "last" => arr.last().map(ValueCow::Borrowed),
                "size" => Some(ValueCow::Owned(Value::scalar(arr.size()))),
                _ => value
                    .as_object()
                    .and_then(|obj| obj.get(index.as_str()).map(ValueCow::Borrowed)),
            }
        }
    } else if let Some(obj) = value.as_object() {
        let lookup = index.is_lookup();
        let index = index.value().to_kstr();
        obj.get(index.as_str()).map(ValueCow::Borrowed).or_else(|| {
            if lookup {
                None
            } else {
                match index.as_str() {
                    "size" => Some(ValueCow::Owned(Value::scalar(obj.size()))),
                    _ => None,
                }
            }
        })
    } else if let Some(scalar) = value.as_scalar() {
        if index.is_lookup() {
            None
        } else {
            let index = index.value().to_kstr();
            match index.as_str() {
                "size" => Some(ValueCow::Owned(Value::scalar(
                    scalar.to_kstr().as_str().len() as i64,
                ))),
                _ => None,
            }
        }
    } else {
        None
    }
}

/// Find a `ValueView` nested in an `ObjectView`
pub fn find<'o>(value: &'o dyn ValueView, path: &[PathElement<'_>]) -> Result<ValueCow<'o>> {
    if let Some(res) = try_find(value, path) {
        Ok(res)
    } else {
        for cur_idx in 1..path.len() {
            let subpath_end = path.len() - cur_idx;
            let subpath = &path[0..subpath_end];
            if let Some(parent) = try_find(value, subpath) {
                let subpath =
                    itertools::join(subpath.iter().map(|index| index.value().render()), ".");
                let requested = &path[subpath_end];
                let available = if let Some(arr) = parent.as_array() {
                    let mut available = vec![
                        KStringCow::from_static("first"),
                        KStringCow::from_static("last"),
                    ];
                    if 0 < arr.size() {
                        available
                            .insert(0, KStringCow::from_string(format!("0..{}", arr.size() - 1)));
                    }
                    available
                } else if let Some(obj) = parent.as_object() {
                    let available: Vec<_> = obj.keys().collect();
                    available
                } else {
                    Vec::new()
                };
                let available = itertools::join(available.iter(), ", ");
                return Error::with_msg("Unknown index")
                    .context("variable", subpath)
                    .context("requested index", format!("{}", requested.value().render()))
                    .context("available indexes", available)
                    .into_err();
            }
        }

        panic!(
            "Should have already errored for `{}` with path {:?}",
            value.source(),
            path
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ValueViewCmp;

    #[test]
    fn array_dot_access_still_supports_first() {
        let value: Value = serde_yaml::from_str(
            r#"
items: [pass]
"#,
        )
        .unwrap();

        let path = [
            PathElement::from("items"),
            PathElement::new("first", PathAccess::Dot),
        ];

        let actual = find(&value, &path).unwrap();
        assert_eq!(actual, ValueViewCmp::new(&"pass"));
    }

    #[test]
    fn array_bracket_string_does_not_alias_first() {
        let value: Value = serde_yaml::from_str(
            r#"
items: [pass]
"#,
        )
        .unwrap();

        let path = [
            PathElement::from("items"),
            PathElement::new("first", PathAccess::Lookup),
        ];

        assert!(try_find(&value, &path).is_none());
    }
}
