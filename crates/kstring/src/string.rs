use std::{borrow::Cow, fmt};

use crate::fixed::*;
use crate::KStringCow;
use crate::KStringRef;

type StdString = std::string::String;
type BoxedStr = Box<str>;

/// A UTF-8 encoded, immutable string.
#[derive(Clone)]
#[repr(transparent)]
pub struct KString {
    pub(crate) inner: KStringInner,
}

#[derive(Clone, Debug)]
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
    Fixed9(FixedString9),
    Fixed10(FixedString10),
    Fixed11(FixedString11),
    Fixed12(FixedString12),
    Fixed13(FixedString13),
    Fixed14(FixedString14),
    Fixed15(FixedString15),
    Fixed16(FixedString16),
}

impl KString {
    /// Create a new empty `KString`.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create an owned `KString`.
    #[inline]
    pub fn from_boxed(other: BoxedStr) -> Self {
        Self {
            inner: KStringInner::Owned(other),
        }
    }

    /// Create an owned `KString`.
    #[inline]
    pub fn from_string(other: StdString) -> Self {
        Self::from_boxed(other.into_boxed_str())
    }

    /// Create an owned `KString` optimally from a reference.
    #[inline]
    pub fn from_ref(other: &str) -> Self {
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
            9 => KStringInner::Fixed9(FixedString9::new(other)),
            10 => KStringInner::Fixed10(FixedString10::new(other)),
            11 => KStringInner::Fixed11(FixedString11::new(other)),
            12 => KStringInner::Fixed12(FixedString12::new(other)),
            13 => KStringInner::Fixed13(FixedString13::new(other)),
            14 => KStringInner::Fixed14(FixedString14::new(other)),
            15 => KStringInner::Fixed15(FixedString15::new(other)),
            16 => KStringInner::Fixed16(FixedString16::new(other)),
            _ => KStringInner::Owned(BoxedStr::from(other)),
        };
        Self { inner }
    }

    /// Create a reference to a `'static` data.
    #[inline]
    pub fn from_static(other: &'static str) -> Self {
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
    pub fn into_string(self) -> StdString {
        String::from(self.into_boxed_str())
    }

    /// Convert to a mutable string type, cloning the data if necessary.
    #[inline]
    pub fn into_boxed_str(self) -> BoxedStr {
        self.inner.into_boxed_str()
    }

    /// Convert to a Cow str
    #[inline]
    pub fn into_cow_str(self) -> Cow<'static, str> {
        self.inner.into_cow_str()
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
            Self::Fixed9(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed10(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed11(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed12(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed13(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed14(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed15(ref s) => KStringRef::from_ref(s.as_str()),
            Self::Fixed16(ref s) => KStringRef::from_ref(s.as_str()),
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
            Self::Fixed9(ref s) => s.as_str(),
            Self::Fixed10(ref s) => s.as_str(),
            Self::Fixed11(ref s) => s.as_str(),
            Self::Fixed12(ref s) => s.as_str(),
            Self::Fixed13(ref s) => s.as_str(),
            Self::Fixed14(ref s) => s.as_str(),
            Self::Fixed15(ref s) => s.as_str(),
            Self::Fixed16(ref s) => s.as_str(),
        }
    }

    #[inline]
    fn into_boxed_str(self) -> BoxedStr {
        match self {
            Self::Owned(s) => s,
            Self::Singleton(s) => BoxedStr::from(s),
            Self::Fixed1(s) => s.to_boxed_str(),
            Self::Fixed2(s) => s.to_boxed_str(),
            Self::Fixed3(s) => s.to_boxed_str(),
            Self::Fixed4(s) => s.to_boxed_str(),
            Self::Fixed5(s) => s.to_boxed_str(),
            Self::Fixed6(s) => s.to_boxed_str(),
            Self::Fixed7(s) => s.to_boxed_str(),
            Self::Fixed8(s) => s.to_boxed_str(),
            Self::Fixed9(s) => s.to_boxed_str(),
            Self::Fixed10(s) => s.to_boxed_str(),
            Self::Fixed11(s) => s.to_boxed_str(),
            Self::Fixed12(s) => s.to_boxed_str(),
            Self::Fixed13(s) => s.to_boxed_str(),
            Self::Fixed14(s) => s.to_boxed_str(),
            Self::Fixed15(s) => s.to_boxed_str(),
            Self::Fixed16(s) => s.to_boxed_str(),
        }
    }

    /// Convert to a Cow str
    #[inline]
    fn into_cow_str(self) -> Cow<'static, str> {
        match self {
            Self::Owned(s) => Cow::Owned(s.into()),
            Self::Singleton(s) => Cow::Borrowed(s),
            Self::Fixed1(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed2(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed3(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed4(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed5(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed6(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed7(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed8(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed9(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed10(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed11(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed12(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed13(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed14(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed15(s) => Cow::Owned(s.to_boxed_str().into()),
            Self::Fixed16(s) => Cow::Owned(s.to_boxed_str().into()),
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

impl fmt::Debug for KString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
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

impl serde::Serialize for KString {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for KString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(StringVisitor)
    }
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = KString;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(KString::from_ref(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(KString::from_string(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match std::str::from_utf8(v) {
            Ok(s) => Ok(KString::from_ref(s)),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(v),
                &self,
            )),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(KString::from_string(s)),
            Err(e) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(&e.into_bytes()),
                &self,
            )),
        }
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
