use std::fmt;

use crate::SString;
use crate::SStringInner;
use crate::SStringRef;
use crate::SStringRefInner;

type StdString = std::string::String;

/// A reference to a UTF-8 encoded, immutable string.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SStringCow<'s> {
    #[serde(with = "serde_string_cow")]
    pub(crate) inner: SStringCowInner<'s>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SStringCowInner<'s> {
    Owned(StdString),
    Borrowed(&'s str),
    Singleton(&'static str),
}

impl<'s> SStringCow<'s> {
    /// Create a new empty `SString`.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create an owned `SString`.
    #[inline]
    pub fn owned(other: impl Into<SString>) -> Self {
        let other = other.into();
        other.into()
    }

    /// Create a reference to a borrowed data.
    #[inline]
    pub fn borrow(other: impl Into<SStringRef<'s>>) -> Self {
        let other = other.into();
        other.into()
    }

    /// Create a reference to a `'static` data.
    #[inline]
    pub fn singleton(other: &'static str) -> Self {
        Self {
            inner: SStringCowInner::Singleton(other),
        }
    }

    /// Get a reference to the `SString`.
    #[inline]
    pub fn as_ref(&self) -> SStringRef<'_> {
        match self.inner {
            SStringCowInner::Owned(ref s) => SStringRef::borrow(s),
            SStringCowInner::Borrowed(ref s) => SStringRef::borrow(*s),
            SStringCowInner::Singleton(ref s) => SStringRef::singleton(s),
        }
    }

    /// Clone the data into an owned-type.
    #[inline]
    pub fn into_owned(self) -> SString {
        match self.inner {
            SStringCowInner::Owned(s) => s.into(),
            SStringCowInner::Borrowed(s) => s.to_owned().into(),
            SStringCowInner::Singleton(s) => s.into(),
        }
    }

    /// Extracts a string slice containing the entire `SStringCow`.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self.inner {
            SStringCowInner::Owned(ref s) => s.as_str(),
            SStringCowInner::Borrowed(ref s) => s,
            SStringCowInner::Singleton(ref s) => s,
        }
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    pub fn into_mut(self) -> StdString {
        match self.inner {
            SStringCowInner::Owned(s) => s,
            SStringCowInner::Borrowed(s) => s.to_owned(),
            SStringCowInner::Singleton(s) => s.to_owned(),
        }
    }
}

impl<'s> std::ops::Deref for SStringCow<'s> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> Eq for SStringCow<'s> {}

impl<'s> PartialEq<SStringCow<'s>> for SStringCow<'s> {
    #[inline]
    fn eq(&self, other: &SStringCow<'s>) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> PartialEq<str> for SStringCow<'s> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for SStringCow<'s> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'s> PartialEq<String> for SStringCow<'s> {
    #[inline]
    fn eq(&self, other: &StdString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> Ord for SStringCow<'s> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'s> PartialOrd for SStringCow<'s> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'s> std::hash::Hash for SStringCow<'s> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'s> fmt::Display for SStringCow<'s> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<'s> AsRef<str> for SStringCow<'s> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> AsRef<[u8]> for SStringCow<'s> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'s> AsRef<std::ffi::OsStr> for SStringCow<'s> {
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        (&**self).as_ref()
    }
}

impl<'s> AsRef<std::path::Path> for SStringCow<'s> {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self)
    }
}

impl<'s> std::borrow::Borrow<str> for SStringCow<'s> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'s> Default for SStringCow<'s> {
    #[inline]
    fn default() -> Self {
        Self::singleton("")
    }
}

impl<'s> From<SString> for SStringCow<'s> {
    #[inline]
    fn from(other: SString) -> Self {
        match other.inner {
            SStringInner::Owned(s) => s.into(),
            SStringInner::Singleton(s) => SStringCow::singleton(s),
        }
    }
}

impl<'s> From<SStringRef<'s>> for SStringCow<'s> {
    #[inline]
    fn from(other: SStringRef<'s>) -> Self {
        match other.inner {
            SStringRefInner::Borrowed(s) => s.into(),
            SStringRefInner::Singleton(s) => SStringCow::singleton(s),
        }
    }
}

impl<'s> From<&'s SStringRef<'s>> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s SStringRef<'s>) -> Self {
        match other.inner {
            SStringRefInner::Borrowed(s) => s.into(),
            SStringRefInner::Singleton(s) => SStringCow::singleton(s),
        }
    }
}

impl<'s> From<StdString> for SStringCow<'s> {
    #[inline]
    fn from(other: StdString) -> Self {
        SStringCow {
            inner: SStringCowInner::Owned(other),
        }
    }
}

impl<'s> From<&'s StdString> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        SStringCow {
            inner: SStringCowInner::Borrowed(other.as_str()),
        }
    }
}

impl<'s> From<&'s str> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s str) -> Self {
        SStringCow {
            inner: SStringCowInner::Borrowed(other),
        }
    }
}

mod serde_string_cow {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(data: &SStringCowInner, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match data {
            SStringCowInner::Owned(ref s) => s.as_str(),
            SStringCowInner::Borrowed(ref s) => s,
            SStringCowInner::Singleton(ref s) => s,
        };
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<SStringCowInner<'static>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = StdString::deserialize(deserializer)?;
        Ok(SStringCowInner::Owned(s))
    }
}
