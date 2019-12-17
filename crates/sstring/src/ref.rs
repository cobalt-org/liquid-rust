use std::fmt;

use crate::SString;
use crate::SStringCow;

type StdString = std::string::String;

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SStringRef<'s> {
    pub(crate) inner: SStringRefInner<'s>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SStringRefInner<'s> {
    Borrowed(&'s str),
    Singleton(&'static str),
}

impl<'s> SStringRef<'s> {
    /// Create a new empty `SString`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a reference to a borrowed data.
    pub fn borrow(other: impl Into<SStringRef<'s>>) -> Self {
        other.into()
    }

    /// Create a reference to a `'static` data.
    pub fn singleton(other: &'static str) -> Self {
        Self {
            inner: SStringRefInner::Singleton(other),
        }
    }

    /// Clone the data into an owned-type.
    pub fn to_owned(&self) -> SString {
        match self.inner {
            SStringRefInner::Borrowed(s) => s.to_owned().into(),
            SStringRefInner::Singleton(s) => s.into(),
        }
    }

    /// Extracts a string slice containing the entire `SStringRef`.
    pub fn as_str(&self) -> &str {
        match self.inner {
            SStringRefInner::Borrowed(ref s) => s,
            SStringRefInner::Singleton(ref s) => s,
        }
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    pub fn into_mut(self) -> StdString {
        match self.inner {
            SStringRefInner::Borrowed(s) => s.to_owned(),
            SStringRefInner::Singleton(s) => s.to_owned(),
        }
    }
}

impl<'s> std::ops::Deref for SStringRef<'s> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> Eq for SStringRef<'s> {}

impl<'s> PartialEq<SStringRef<'s>> for SStringRef<'s> {
    #[inline]
    fn eq(&self, other: &SStringRef<'s>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> PartialEq<str> for SStringRef<'s> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for SStringRef<'s> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'s> PartialEq<String> for SStringRef<'s> {
    #[inline]
    fn eq(&self, other: &StdString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> Ord for SStringRef<'s> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'s> PartialOrd for SStringRef<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'s> std::hash::Hash for SStringRef<'s> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'s> fmt::Display for SStringRef<'s> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<'s> AsRef<str> for SStringRef<'s> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> AsRef<[u8]> for SStringRef<'s> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'s> AsRef<std::ffi::OsStr> for SStringRef<'s> {
    fn as_ref(&self) -> &std::ffi::OsStr {
        (&**self).as_ref()
    }
}

impl<'s> AsRef<std::path::Path> for SStringRef<'s> {
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self)
    }
}

impl<'s> std::borrow::Borrow<str> for SStringRef<'s> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'s> Default for SStringRef<'s> {
    fn default() -> Self {
        Self::singleton("")
    }
}

impl<'s> From<&'s SString> for SStringRef<'s> {
    fn from(other: &'s SString) -> Self {
        other.as_ref()
    }
}

impl<'s> From<&'s SStringCow<'s>> for SStringRef<'s> {
    fn from(other: &'s SStringCow<'s>) -> Self {
        other.as_ref()
    }
}

impl<'s> From<&'s StdString> for SStringRef<'s> {
    fn from(other: &'s StdString) -> Self {
        SStringRef {
            inner: SStringRefInner::Borrowed(other.as_str()),
        }
    }
}

impl<'s> From<&'s str> for SStringRef<'s> {
    fn from(other: &'s str) -> Self {
        SStringRef {
            inner: SStringRefInner::Borrowed(other),
        }
    }
}
