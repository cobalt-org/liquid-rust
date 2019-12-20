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
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a reference to a borrowed data.
    #[inline]
    pub fn borrow(other: impl Into<SStringRef<'s>>) -> Self {
        other.into()
    }

    /// Create a reference to a `'static` data.
    #[inline]
    pub fn singleton(other: &'static str) -> Self {
        Self::from_static(other)
    }

    #[inline]
    pub(crate) fn from_ref(other: &'s str) -> Self {
        Self {
            inner: SStringRefInner::Borrowed(other),
        }
    }

    #[inline]
    pub(crate) fn from_static(other: &'static str) -> Self {
        Self {
            inner: SStringRefInner::Singleton(other),
        }
    }

    /// Clone the data into an owned-type.
    #[inline]
    pub fn to_owned(&self) -> SString {
        self.inner.to_owned()
    }

    /// Extracts a string slice containing the entire `SStringRef`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    pub fn into_mut(self) -> StdString {
        self.inner.into_mut()
    }
}

impl<'s> SStringRefInner<'s> {
    #[inline]
    fn to_owned(&self) -> SString {
        match self {
            Self::Borrowed(s) => SString::from_ref(s),
            Self::Singleton(s) => SString::from_static(s),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(ref s) => s,
            Self::Singleton(ref s) => s,
        }
    }

    #[inline]
    fn into_mut(self) -> StdString {
        self.as_str().to_owned()
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
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'s> PartialOrd for SStringRef<'s> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'s> std::hash::Hash for SStringRef<'s> {
    #[inline]
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
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        (&**self).as_ref()
    }
}

impl<'s> AsRef<std::path::Path> for SStringRef<'s> {
    #[inline]
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
    #[inline]
    fn default() -> Self {
        Self::from_static("")
    }
}

impl<'s> From<&'s SString> for SStringRef<'s> {
    #[inline]
    fn from(other: &'s SString) -> Self {
        other.as_ref()
    }
}

impl<'s> From<&'s SStringCow<'s>> for SStringRef<'s> {
    #[inline]
    fn from(other: &'s SStringCow<'s>) -> Self {
        other.as_ref()
    }
}

impl<'s> From<&'s StdString> for SStringRef<'s> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        SStringRef::from_ref(other.as_str())
    }
}

impl<'s> From<&'s str> for SStringRef<'s> {
    #[inline]
    fn from(other: &'s str) -> Self {
        SStringRef::from_ref(other)
    }
}
