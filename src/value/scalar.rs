use std::cmp::Ordering;
use std::fmt;
use std::borrow;

use chrono;

pub type Date = chrono::DateTime<chrono::FixedOffset>;

/// A Liquid scalar value
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scalar(ScalarEnum);

/// An enum to represent different value types
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
enum ScalarEnum {
    Integer(i32),
    Float(f32),
    Bool(bool),
    #[cfg_attr(feature = "serde", serde(with = "friendly_date"))]
    Date(Date),
    Str(String),
}

impl Scalar {
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }

    pub fn to_str(&self) -> borrow::Cow<str> {
        match self.0 {
            ScalarEnum::Integer(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarEnum::Float(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarEnum::Bool(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarEnum::Date(ref x) => borrow::Cow::Owned(x.format(DATE_FORMAT).to_string()),
            ScalarEnum::Str(ref x) => borrow::Cow::Borrowed(x.as_str()),
        }
    }

    pub fn into_string(self) -> String {
        match self.0 {
            ScalarEnum::Integer(x) => x.to_string(),
            ScalarEnum::Float(x) => x.to_string(),
            ScalarEnum::Bool(x) => x.to_string(),
            ScalarEnum::Date(x) => x.to_string(),
            ScalarEnum::Str(x) => x,
        }
    }

    /// Interpret as an integer, if possible
    pub fn to_integer(&self) -> Option<i32> {
        match self.0 {
            ScalarEnum::Integer(ref x) => Some(*x),
            ScalarEnum::Str(ref x) => x.parse::<i32>().ok(),
            _ => None,
        }
    }

    /// Interpret as an float, if possible
    pub fn to_float(&self) -> Option<f32> {
        match self.0 {
            ScalarEnum::Integer(ref x) => Some(*x as f32),
            ScalarEnum::Float(ref x) => Some(*x),
            ScalarEnum::Str(ref x) => x.parse::<f32>().ok(),
            _ => None,
        }
    }

    /// Interpret as an bool, if possible
    pub fn to_bool(&self) -> Option<bool> {
        match self.0 {
            ScalarEnum::Bool(ref x) => Some(*x),
            _ => None,
        }
    }

    /// Interpret as an bool, if possible
    pub fn to_date(&self) -> Option<Date> {
        match self.0 {
            ScalarEnum::Date(ref x) => Some(*x),
            ScalarEnum::Str(ref x) => parse_date(x.as_str()),
            _ => None,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_truthy(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match self.0 {
            ScalarEnum::Bool(ref x) => *x,
            _ => true,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_default(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match self.0 {
            ScalarEnum::Bool(ref x) => !*x,
            ScalarEnum::Str(ref x) => x.is_empty(),
            _ => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.0 {
            ScalarEnum::Integer(_) => "whole number",
            ScalarEnum::Float(_) => "fractional number",
            ScalarEnum::Bool(_) => "boolean",
            ScalarEnum::Date(_) => "date",
            ScalarEnum::Str(_) => "string",
        }
    }
}

impl From<i32> for Scalar {
    fn from(s: i32) -> Self {
        Scalar { 0: ScalarEnum::Integer(s) }
    }
}

impl From<f32> for Scalar {
    fn from(s: f32) -> Self {
        Scalar { 0: ScalarEnum::Float(s) }
    }
}

impl From<bool> for Scalar {
    fn from(s: bool) -> Self {
        Scalar { 0: ScalarEnum::Bool(s) }
    }
}

impl From<Date> for Scalar {
    fn from(s: Date) -> Self {
        Scalar { 0: ScalarEnum::Date(s) }
    }
}

impl From<String> for Scalar {
    fn from(s: String) -> Self {
        Scalar { 0: ScalarEnum::Str(s) }
    }
}

impl<'a> From<&'a String> for Scalar {
    fn from(s: &String) -> Self {
        Scalar { 0: ScalarEnum::Str(s.to_owned()) }
    }
}

impl<'a> From<&'a str> for Scalar {
    fn from(s: &str) -> Self {
        Scalar { 0: ScalarEnum::Str(s.to_owned()) }
    }
}

impl PartialEq<Scalar> for Scalar {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (&ScalarEnum::Integer(x), &ScalarEnum::Integer(y)) => x == y,
            (&ScalarEnum::Integer(x), &ScalarEnum::Float(y)) => (x as f32) == y,
            (&ScalarEnum::Float(x), &ScalarEnum::Integer(y)) => x == (y as f32),
            (&ScalarEnum::Float(x), &ScalarEnum::Float(y)) => x == y,
            (&ScalarEnum::Bool(x), &ScalarEnum::Bool(y)) => x == y,
            (&ScalarEnum::Date(x), &ScalarEnum::Date(y)) => x == y,
            (&ScalarEnum::Str(ref x), &ScalarEnum::Str(ref y)) => x == y,
            // encode Ruby truthiness: all values except false and nil are true
            (_, &ScalarEnum::Bool(b)) |
            (&ScalarEnum::Bool(b), _) => b,
            _ => false,
        }
    }
}

impl Eq for Scalar {}

impl PartialOrd<Scalar> for Scalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (&ScalarEnum::Integer(x), &ScalarEnum::Integer(y)) => x.partial_cmp(&y),
            (&ScalarEnum::Integer(x), &ScalarEnum::Float(y)) => (x as f32).partial_cmp(&y),
            (&ScalarEnum::Float(x), &ScalarEnum::Integer(y)) => x.partial_cmp(&(y as f32)),
            (&ScalarEnum::Float(x), &ScalarEnum::Float(y)) => x.partial_cmp(&y),
            (&ScalarEnum::Bool(x), &ScalarEnum::Bool(y)) => x.partial_cmp(&y),
            (&ScalarEnum::Date(x), &ScalarEnum::Date(y)) => x.partial_cmp(&y),
            (&ScalarEnum::Str(ref x), &ScalarEnum::Str(ref y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.to_str();
        write!(f, "{}", data)
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

#[cfg(feature = "serde")]
mod friendly_date {
    use super::*;
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let s = date.format(DATE_FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Date::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
    }
}

fn parse_date(s: &str) -> Option<Date> {
    let formats = ["%d %B %Y %H:%M:%S %z", "%Y-%m-%d %H:%M:%S %z"];
    formats
        .iter()
        .filter_map(|f| Date::parse_from_str(s, f).ok())
        .next()
}

#[cfg(test)]
mod test {
    use super::*;

    static TRUE: Scalar = Scalar(ScalarEnum::Bool(true));
    static FALSE: Scalar = Scalar(ScalarEnum::Bool(false));

    #[test]
    fn test_to_str_bool() {
        assert_eq!(TRUE.to_str(), "true");
    }

    #[test]
    fn test_to_str_integer() {
        let val: Scalar = 42i32.into();
        assert_eq!(val.to_str(), "42");
    }

    #[test]
    fn test_to_str_float() {
        let val: Scalar = 42f32.into();
        assert_eq!(val.to_str(), "42");

        let val: Scalar = 42.34.into();
        assert_eq!(val.to_str(), "42.34");
    }

    #[test]
    fn test_to_str_str() {
        let val: Scalar = "foobar".into();
        assert_eq!(val.to_str(), "foobar");
    }

    #[test]
    fn test_to_integer_bool() {
        assert_eq!(TRUE.to_integer(), None);
    }

    #[test]
    fn test_to_integer_integer() {
        let val: Scalar = 42i32.into();
        assert_eq!(val.to_integer(), Some(42i32));
    }

    #[test]
    fn test_to_integer_float() {
        let val: Scalar = 42f32.into();
        assert_eq!(val.to_integer(), None);

        let val: Scalar = 42.34.into();
        assert_eq!(val.to_integer(), None);
    }

    #[test]
    fn test_to_integer_str() {
        let val: Scalar = "foobar".into();
        assert_eq!(val.to_integer(), None);

        let val: Scalar = "42.34".into();
        assert_eq!(val.to_integer(), None);

        let val: Scalar = "42".into();
        assert_eq!(val.to_integer(), Some(42));
    }

    #[test]
    fn test_to_float_bool() {
        assert_eq!(TRUE.to_float(), None);
    }

    #[test]
    fn test_to_float_integer() {
        let val: Scalar = 42i32.into();
        assert_eq!(val.to_float(), Some(42f32));
    }

    #[test]
    fn test_to_float_float() {
        let val: Scalar = 42f32.into();
        assert_eq!(val.to_float(), Some(42f32));

        let val: Scalar = 42.34.into();
        assert_eq!(val.to_float(), Some(42.34));
    }

    #[test]
    fn test_to_float_str() {
        let val: Scalar = "foobar".into();
        assert_eq!(val.to_float(), None);

        let val: Scalar = "42.34".into();
        assert_eq!(val.to_float(), Some(42.34));

        let val: Scalar = "42".into();
        assert_eq!(val.to_float(), Some(42f32));
    }

    #[test]
    fn test_to_bool_bool() {
        assert_eq!(TRUE.to_bool(), Some(true));
    }

    #[test]
    fn test_to_bool_integer() {
        let val: Scalar = 42i32.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_float() {
        let val: Scalar = 42f32.into();
        assert_eq!(val.to_bool(), None);

        let val: Scalar = 42.34.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_str() {
        let val: Scalar = "foobar".into();
        assert_eq!(val.to_bool(), None);

        let val: Scalar = "true".into();
        assert_eq!(val.to_bool(), None);

        let val: Scalar = "false".into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn integer_equality() {
        let val: Scalar = 42i32.into();
        let zero: Scalar = 0i32.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn integers_have_ruby_truthiness() {
        let val: Scalar = 42i32.into();
        let zero: Scalar = 0i32.into();
        assert_eq!(TRUE, val);
        assert_eq!(val, TRUE);
        assert!(val.is_truthy());

        assert_eq!(TRUE, zero);
        assert_eq!(zero, TRUE);
        assert!(zero.is_truthy());
    }

    #[test]
    fn float_equality() {
        let val: Scalar = 42f32.into();
        let zero: Scalar = 0f32.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn floats_have_ruby_truthiness() {
        let val: Scalar = 42f32.into();
        let zero: Scalar = 0f32.into();
        assert_eq!(TRUE, val);
        assert_eq!(val, TRUE);
        assert!(val.is_truthy());

        assert_eq!(TRUE, zero);
        assert_eq!(zero, TRUE);
        assert!(zero.is_truthy());
    }

    #[test]
    fn boolean_equality() {
        assert_eq!(TRUE, TRUE);
        assert_eq!(FALSE, FALSE);
        assert!(FALSE != TRUE);
        assert!(TRUE != FALSE);
    }

    #[test]
    fn booleans_have_ruby_truthiness() {
        assert!(TRUE.is_truthy());
        assert!(!FALSE.is_truthy());
    }

    #[test]
    fn string_equality() {
        let alpha: Scalar = "alpha".into();
        let beta: Scalar = "beta".into();
        let empty: Scalar = "".into();
        assert_eq!(alpha, alpha);
        assert_eq!(empty, empty);
        assert!(alpha != beta);
        assert!(beta != alpha);
    }

    #[test]
    fn strings_have_ruby_truthiness() {
        // all strings in ruby are true
        let alpha: Scalar = "alpha".into();
        let empty: Scalar = "".into();
        assert_eq!(TRUE, alpha);
        assert_eq!(alpha, TRUE);
        assert!(alpha.is_truthy());

        assert_eq!(TRUE, empty);
        assert_eq!(empty, TRUE);
        assert!(empty.is_truthy());
    }
}
