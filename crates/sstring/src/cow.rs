use std::fmt;

use crate::SString;
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
    Owned(SString),
    Borrowed(&'s str),
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
        // TODO: Used fixed strings
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
        Self::from_static(other)
    }

    #[inline]
    pub(crate) fn from_string(other: String) -> Self {
        Self {
            inner: SStringCowInner::Owned(SString::from_string(other)),
        }
    }

    #[inline]
    pub(crate) fn from_ref(other: &'s str) -> Self {
        Self {
            inner: SStringCowInner::Borrowed(other),
        }
    }

    #[inline]
    pub(crate) fn from_static(other: &'static str) -> Self {
        Self {
            inner: SStringCowInner::Owned(SString::from_static(other)),
        }
    }

    /// Get a reference to the `SString`.
    #[inline]
    pub fn as_ref(&self) -> SStringRef<'_> {
        self.inner.as_ref()
    }

    /// Clone the data into an owned-type.
    #[inline]
    pub fn into_owned(self) -> SString {
        self.inner.into_owned()
    }

    /// Extracts a string slice containing the entire `SStringCow`.
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

impl<'s> SStringCowInner<'s> {
    #[inline]
    fn as_ref(&self) -> SStringRef<'_> {
        match self {
            Self::Owned(ref s) => s.as_ref(),
            Self::Borrowed(ref s) => SStringRef::from_ref(s),
        }
    }

    #[inline]
    fn into_owned(self) -> SString {
        match self {
            Self::Owned(s) => s,
            Self::Borrowed(s) => SString::from_ref(s),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Owned(ref s) => s.as_str(),
            Self::Borrowed(ref s) => s,
        }
    }

    #[inline]
    fn into_mut(self) -> StdString {
        match self {
            Self::Owned(s) => s.into_mut(),
            Self::Borrowed(s) => s.to_owned(),
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
        Self::from_static("")
    }
}

impl From<SString> for SStringCow<'static> {
    #[inline]
    fn from(other: SString) -> Self {
        let inner = SStringCowInner::Owned(other);
        Self { inner }
    }
}

impl<'s> From<SStringRef<'s>> for SStringCow<'s> {
    #[inline]
    fn from(other: SStringRef<'s>) -> Self {
        match other.inner {
            SStringRefInner::Borrowed(s) => Self::from_ref(s),
            SStringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl<'s> From<&'s SStringRef<'s>> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s SStringRef<'s>) -> Self {
        match other.inner {
            SStringRefInner::Borrowed(s) => Self::from_ref(s),
            SStringRefInner::Singleton(s) => Self::from_static(s),
        }
    }
}

impl From<StdString> for SStringCow<'static> {
    #[inline]
    fn from(other: StdString) -> Self {
        // Since the memory is already allocated, don't bother moving it into a FixedString
        Self::from_string(other)
    }
}

impl<'s> From<&'s StdString> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s StdString) -> Self {
        Self::from_ref(other.as_str())
    }
}

impl<'s> From<&'s str> for SStringCow<'s> {
    #[inline]
    fn from(other: &'s str) -> Self {
        Self::from_ref(other)
    }
}

mod serde_string_cow {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(data: &SStringCowInner, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&data.as_str())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<SStringCowInner<'static>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::borrow::Cow;
        let s: Cow<'_, str> = Cow::deserialize(deserializer)?;
        let s = match s {
            Cow::Owned(s) => SStringCow::from_string(s),
            Cow::Borrowed(s) => SStringCow::from_string(s.to_owned()),
        };
        Ok(s.inner)
    }
}
