use std::fmt;

use crate::FixedString1;
use crate::FixedString2;
use crate::FixedString3;
use crate::FixedString4;
use crate::FixedString5;
use crate::FixedString6;
use crate::FixedString7;
use crate::FixedString8;
use crate::KStringCow;
use crate::KStringRef;

type StdString = std::string::String;
type BoxedStr = Box<str>;

/// A UTF-8 encoded, immutable string.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KString {
    #[serde(with = "serde_string")]
    pub(crate) inner: KStringInner,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum KStringInner {
    Owned(BoxedStr),
    Singleton(&'static str),
    Fixed1(FixedString1),
    Fixed2(FixedString2),
    Fixed3(FixedString3),
    Fixed4(FixedString4),
    Fixed5(FixedString5),
    Fixed6(FixedString6),
    Fixed7(FixedString7),
    Fixed8(FixedString8),
}

impl KString {
    /// Create a new empty `KString`.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create an owned `KString`.
    #[inline]
    pub fn owned(other: impl Into<StdString>) -> Self {
        // TODO: Used fixed strings
        Self::from_boxed(other.into().into_boxed_str())
    }

    /// Create a reference to a `'static` data.
    #[inline]
    pub fn singleton(other: &'static str) -> Self {
        Self::from_static(other)
    }

    #[inline]
    pub(crate) fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: KStringInner::Owned(other),
        }
    }

    #[inline]
    pub(crate) fn from_ref(other: &str) -> Self {
        let inner = match other.len() {
            0 => KStringInner::Singleton(""),
            1 => KStringInner::Fixed1(FixedString1::new(other)),
            2 => KStringInner::Fixed2(FixedString2::new(other)),
            3 => KStringInner::Fixed3(FixedString3::new(other)),
            4 => KStringInner::Fixed4(FixedString4::new(other)),
            5 => KStringInner::Fixed5(FixedString5::new(other)),
            6 => KStringInner::Fixed6(FixedString6::new(other)),
            7 => KStringInner::Fixed7(FixedString7::new(other)),
            8 => KStringInner::Fixed8(FixedString8::new(other)),
            _ => KStringInner::Owned(other.to_owned().into_boxed_str()),
        };
        Self { inner }
    }

    #[inline]
    pub(crate) fn from_static(other: &'static str) -> Self {
        Self {
            inner: KStringInner::Singleton(other),
        }
    }

    /// Get a reference to the `KString`.
    #[inline]
    pub fn as_ref(&self) -> KStringRef<'_> {
        self.inner.as_ref()
    }

    /// Extracts a string slice containing the entire `KString`.
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

impl KStringInner {
    #[inline]
    fn as_ref(&self) -> KStringRef<'_> {
        match self {
            Self::Owned(ref s) => KStringRef::from_ref(s),
            Self::Singleton(ref s) => KStringRef::from_static(s),
            Self::Fixed1(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed2(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed3(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed4(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed5(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed6(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed7(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed8(ref s) => KStringRef::from_ref(s.as_str()),
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Owned(ref s) => &s,
            Self::Singleton(ref s) => s,
            Self::Fixed1(ref s) => s.as_str(),
            Self::Fixed2(ref s) => s.as_str(),
            Self::Fixed3(ref s) => s.as_str(),
            Self::Fixed4(ref s) => s.as_str(),
            Self::Fixed5(ref s) => s.as_str(),
            Self::Fixed6(ref s) => s.as_str(),
            Self::Fixed7(ref s) => s.as_str(),
            Self::Fixed8(ref s) => s.as_str(),
        }
    }

    #[inline]
    fn into_mut(self) -> StdString {
        match self {
            Self::Owned(s) => String::from(s),
            Self::Singleton(s) => s.to_owned(),
            Self::Fixed1(s) => s.into_mut(),
            Self::Fixed2(s) => s.into_mut(),
            Self::Fixed3(s) => s.into_mut(),
            Self::Fixed4(s) => s.into_mut(),
            Self::Fixed5(s) => s.into_mut(),
            Self::Fixed6(s) => s.into_mut(),
            Self::Fixed7(s) => s.into_mut(),
            Self::Fixed8(s) => s.into_mut(),
        }
    }
}

impl std::ops::Deref for KString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Eq for KString {}

impl<'s> PartialEq<KString> for KString {
    #[inline]
    fn eq(&self, other: &KString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl<'s> PartialEq<str> for KString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'s> PartialEq<&'s str> for KString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl<'s> PartialEq<String> for KString {
    #[inline]
    fn eq(&self, other: &StdString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl Ord for KString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for KString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl std::hash::Hash for KString {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for KString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl AsRef<str> for KString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for KString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<std::ffi::OsStr> for KString {
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        (&**self).as_ref()
    }
}

impl AsRef<std::path::Path> for KString {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        std::path::Path::new(self)
    }
}

impl std::borrow::Borrow<str> for KString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for KString {
    #[inline]
    fn default() -> Self {
        Self::from_static("")
    }
}

impl<'s> From<KStringRef<'s>> for KString {
    #[inline]
    fn from(other: KStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s> From<&'s KStringRef<'s>> for KString {
    #[inline]
    fn from(other: &'s KStringRef<'s>) -> Self {
        other.to_owned()
    }
}

impl<'s> From<KStringCow<'s>> for KString {
    #[inline]
    fn from(other: KStringCow<'s>) -> Self {
        other.into_owned()
    }
}

impl<'s> From<&'s KStringCow<'s>> for KString {
    #[inline]
    fn from(other: &'s KStringCow<'s>) -> Self {
        other.clone().into_owned()
    }
}

impl From<StdString> for KString {
    #[inline]
    fn from(other: StdString) -> Self {
        // Since the memory is already allocated, don't bother moving it into a FixedString
        Self::from_boxed(other.into_boxed_str())
    }
}

impl From<BoxedStr> for KString {
    #[inline]
    fn from(other: BoxedStr) -> Self {
        // Since the memory is already allocated, don't bother moving it into a FixedString
        Self::from_boxed(other)
    }
}

impl<'s> From<&'s BoxedStr> for KString {
    #[inline]
    fn from(other: &'s BoxedStr) -> Self {
        Self::from_ref(other)
    }
}

impl From<&'static str> for KString {
    #[inline]
    fn from(other: &'static str) -> Self {
        Self::from_static(other)
    }
}

mod serde_string {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(data: &KStringInner, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&data.as_str())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<KStringInner, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::borrow::Cow;
        let s: Cow<'_, str> = Cow::deserialize(deserializer)?;
        let s = match s {
            Cow::Owned(s) => KString::from_boxed(s.into_boxed_str()),
            Cow::Borrowed(s) => KString::from_ref(s),
        };
        Ok(s.inner)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_size() {
        println!("String: {}", std::mem::size_of::<StdString>());
        println!("Box<str>: {}", std::mem::size_of::<BoxedStr>());
        println!("Box<Box<str>>: {}", std::mem::size_of::<Box<BoxedStr>>());
        println!("KString: {}", std::mem::size_of::<KString>());
    }
}
