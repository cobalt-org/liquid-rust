use std::fmt;

use crate::SStringCow;
use crate::SStringRef;

type StdString = std::string::String;

/// A UTF-8 encoded, immutable string.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SString {
    #[serde(with = "serde_string")]
    pub(crate) inner: SStringInner,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SStringInner {
    Owned(StdString),
    Singleton(&'static str),
}

impl SString {
    /// Create a new empty `SString`.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create an owned `SString`.
    #[inline]
    pub fn owned(other: impl Into<StdString>) -> Self {
        Self::from_string(other.into())
    }

    /// Create a reference to a `'static` data.
    #[inline]
    pub fn singleton(other: &'static str) -> Self {
        Self::from_static(other)
    }

    #[inline]
    pub(crate) fn from_string(other: String) -> Self {
        Self {
            inner: SStringInner::Owned(other),
        }
    }

    #[inline]
    pub(crate) fn from_ref(other: &str) -> Self {
        Self::from_string(other.to_owned())
    }

    #[inline]
    pub(crate) fn from_static(other: &'static str) -> Self {
        Self {
            inner: SStringInner::Singleton(other),
        }
    }

    /// Get a reference to the `SString`.
    #[inline]
    pub fn as_ref(&self) -> SStringRef<'_> {
        self.inner.as_ref()
    }

    /// Extracts a string slice containing the entire `SString`.
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

impl SStringInner {
    #[inline]
    fn as_ref(&self) -> SStringRef<'_> {
        match self {
            SStringInner::Owned(ref s) => SStringRef::from_ref(s),
            SStringInner::Singleton(ref s) => SStringRef::from_static(s),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            SStringInner::Owned(ref s) => s.as_str(),
            SStringInner::Singleton(ref s) => s,
        }
    }

    #[inline]
    fn into_mut(self) -> StdString {
        match self {
            SStringInner::Owned(s) => s,
            SStringInner::Singleton(s) => s.to_owned(),
        }
    }
}

impl std::ops::Deref for SString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Eq for SString {}

impl<'s> PartialEq<SString> for SString {
    #[inline]
    fn eq(&self, other: &SString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> PartialEq<str> for SString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for SString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'s> PartialEq<String> for SString {
    #[inline]
    fn eq(&self, other: &StdString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl Ord for SString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for SString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl std::hash::Hash for SString {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for SString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for SString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for SString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<std::ffi::OsStr> for SString {
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        (&**self).as_ref()
    }
}

impl AsRef<std::path::Path> for SString {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self)
    }
}

impl std::borrow::Borrow<str> for SString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for SString {
    #[inline]
    fn default() -> Self {
        Self::from_static("")
    }
}

impl<'s> From<SStringRef<'s>> for SString {
    #[inline]
    fn from(other: SStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s> From<&'s SStringRef<'s>> for SString {
    #[inline]
    fn from(other: &'s SStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s> From<SStringCow<'s>> for SString {
    #[inline]
    fn from(other: SStringCow<'s>) -> Self {
        other.into_owned()
    }
}

impl<'s> From<&'s SStringCow<'s>> for SString {
    #[inline]
    fn from(other: &'s SStringCow<'s>) -> Self {
        other.clone().into_owned()
    }
}

impl From<StdString> for SString {
    #[inline]
    fn from(other: StdString) -> Self {
        Self::from_string(other)
    }
}

impl From<&'static str> for SString {
    #[inline]
    fn from(other: &'static str) -> Self {
        Self::from_static(other)
    }
}

mod serde_string {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(data: &SStringInner, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&data.as_str())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<SStringInner, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::borrow::Cow;
        let s: Cow<'_, str> = Cow::deserialize(deserializer)?;
        let s = match s {
            Cow::Owned(s) => SString::from_string(s),
            Cow::Borrowed(s) => SString::from_ref(s),
        };
        Ok(s.inner)
    }
}
