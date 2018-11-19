use std::borrow;
use std::cmp::Ordering;
use std::fmt;

use chrono;

/// Liquid's native date/time type.
pub type Date = chrono::DateTime<chrono::FixedOffset>;

/// A Liquid scalar value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalarCow<'s>(ScalarCowEnum<'s>);

/// A Liquid scalar value
pub type Scalar = ScalarCow<'static>;

/// An enum to represent different value types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ScalarCowEnum<'s> {
    Integer(i32),
    Float(f64),
    Bool(bool),
    #[serde(with = "friendly_date")]
    Date(Date),
    Str(borrow::Cow<'s, str>),
}

impl<'s> ScalarCow<'s> {
    /// Convert a value into a `ScalarCow`.
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }

    /// Create an owned version of the value.
    pub fn into_owned(self) -> Self {
        match self.0 {
            ScalarCowEnum::Str(x) => Scalar::new(x.into_owned()),
            _ => self,
        }
    }

    /// Create a reference to the value.
    pub fn as_ref<'r: 's>(&'r self) -> ScalarCow<'r> {
        match self.0 {
            ScalarCowEnum::Integer(x) => Scalar::new(x),
            ScalarCowEnum::Float(x) => Scalar::new(x),
            ScalarCowEnum::Bool(x) => Scalar::new(x),
            ScalarCowEnum::Date(x) => Scalar::new(x),
            ScalarCowEnum::Str(ref x) => Scalar::new(x.as_ref()),
        }
    }

    /// Interpret as a string.
    pub fn to_str(&self) -> borrow::Cow<str> {
        match self.0 {
            ScalarCowEnum::Integer(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarCowEnum::Float(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarCowEnum::Bool(ref x) => borrow::Cow::Owned(x.to_string()),
            ScalarCowEnum::Date(ref x) => borrow::Cow::Owned(x.format(DATE_FORMAT).to_string()),
            ScalarCowEnum::Str(ref x) => borrow::Cow::Borrowed(x.as_ref()),
        }
    }

    /// Convert to a string.
    pub fn into_string(self) -> String {
        match self.0 {
            ScalarCowEnum::Integer(x) => x.to_string(),
            ScalarCowEnum::Float(x) => x.to_string(),
            ScalarCowEnum::Bool(x) => x.to_string(),
            ScalarCowEnum::Date(x) => x.to_string(),
            ScalarCowEnum::Str(x) => x.into_owned(),
        }
    }

    /// Interpret as an integer, if possible
    pub fn to_integer(&self) -> Option<i32> {
        match self.0 {
            ScalarCowEnum::Integer(ref x) => Some(*x),
            ScalarCowEnum::Str(ref x) => x.parse::<i32>().ok(),
            _ => None,
        }
    }

    /// Interpret as a float, if possible
    pub fn to_float(&self) -> Option<f64> {
        match self.0 {
            ScalarCowEnum::Integer(ref x) => Some(f64::from(*x)),
            ScalarCowEnum::Float(ref x) => Some(*x),
            ScalarCowEnum::Str(ref x) => x.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Interpret as a bool, if possible
    pub fn to_bool(&self) -> Option<bool> {
        match self.0 {
            ScalarCowEnum::Bool(ref x) => Some(*x),
            _ => None,
        }
    }

    /// Interpret as a date, if possible
    pub fn to_date(&self) -> Option<Date> {
        match self.0 {
            ScalarCowEnum::Date(ref x) => Some(*x),
            ScalarCowEnum::Str(ref x) => parse_date(x.as_ref()),
            _ => None,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_truthy(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match self.0 {
            ScalarCowEnum::Bool(ref x) => *x,
            _ => true,
        }
    }

    /// Whether a default constructed value.
    pub fn is_default(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match self.0 {
            ScalarCowEnum::Bool(ref x) => !*x,
            ScalarCowEnum::Str(ref x) => x.is_empty(),
            _ => false,
        }
    }

    /// Report the data type (generally for error reporting).
    pub fn type_name(&self) -> &'static str {
        match self.0 {
            ScalarCowEnum::Integer(_) => "whole number",
            ScalarCowEnum::Float(_) => "fractional number",
            ScalarCowEnum::Bool(_) => "boolean",
            ScalarCowEnum::Date(_) => "date",
            ScalarCowEnum::Str(_) => "string",
        }
    }
}

impl<'s> From<i32> for ScalarCow<'s> {
    fn from(s: i32) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Integer(s),
        }
    }
}

impl<'s> From<f64> for ScalarCow<'s> {
    fn from(s: f64) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Float(s),
        }
    }
}

impl<'s> From<bool> for ScalarCow<'s> {
    fn from(s: bool) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Bool(s),
        }
    }
}

impl<'s> From<Date> for ScalarCow<'s> {
    fn from(s: Date) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Date(s),
        }
    }
}

impl<'s> From<String> for ScalarCow<'s> {
    fn from(s: String) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.into()),
        }
    }
}

impl<'s> From<&'s String> for ScalarCow<'s> {
    fn from(s: &'s String) -> ScalarCow<'s> {
        ScalarCow {
            0: ScalarCowEnum::Str(s.as_str().into()),
        }
    }
}

impl<'s> From<&'s str> for ScalarCow<'s> {
    fn from(s: &'s str) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.into()),
        }
    }
}

impl<'s> From<borrow::Cow<'s, str>> for ScalarCow<'s> {
    fn from(s: borrow::Cow<'s, str>) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s),
        }
    }
}

impl<'s> PartialEq<ScalarCow<'s>> for ScalarCow<'s> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Integer(y)) => x == y,
            (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Float(y)) => (f64::from(x)) == y,
            (&ScalarCowEnum::Float(x), &ScalarCowEnum::Integer(y)) => x == (f64::from(y)),
            (&ScalarCowEnum::Float(x), &ScalarCowEnum::Float(y)) => x == y,
            (&ScalarCowEnum::Bool(x), &ScalarCowEnum::Bool(y)) => x == y,
            (&ScalarCowEnum::Date(x), &ScalarCowEnum::Date(y)) => x == y,
            (&ScalarCowEnum::Str(ref x), &ScalarCowEnum::Str(ref y)) => x == y,
            // encode Ruby truthiness: all values except false and nil are true
            (_, &ScalarCowEnum::Bool(b)) | (&ScalarCowEnum::Bool(b), _) => b,
            _ => false,
        }
    }
}

impl<'s> Eq for ScalarCow<'s> {}

impl<'s> PartialOrd<ScalarCow<'s>> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Integer(y)) => x.partial_cmp(&y),
            (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Float(y)) => {
                (f64::from(x)).partial_cmp(&y)
            }
            (&ScalarCowEnum::Float(x), &ScalarCowEnum::Integer(y)) => {
                x.partial_cmp(&(f64::from(y)))
            }
            (&ScalarCowEnum::Float(x), &ScalarCowEnum::Float(y)) => x.partial_cmp(&y),
            (&ScalarCowEnum::Bool(x), &ScalarCowEnum::Bool(y)) => x.partial_cmp(&y),
            (&ScalarCowEnum::Date(x), &ScalarCowEnum::Date(y)) => x.partial_cmp(&y),
            (&ScalarCowEnum::Str(ref x), &ScalarCowEnum::Str(ref y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl<'s> fmt::Display for ScalarCow<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.to_str();
        write!(f, "{}", data)
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

mod friendly_date {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(DATE_FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
    }
}

fn parse_date(s: &str) -> Option<Date> {
    match s {
        "now" | "today" => {
            let now = chrono::offset::Utc::now();
            let now = now.naive_utc();
            let now = chrono::DateTime::from_utc(now, chrono::offset::FixedOffset::east(0));
            Some(now)
        }
        _ => {
            let formats = ["%d %B %Y %H:%M:%S %z", "%Y-%m-%d %H:%M:%S %z"];
            formats
                .iter()
                .filter_map(|f| Date::parse_from_str(s, f).ok())
                .next()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static TRUE: ScalarCow = ScalarCow(ScalarCowEnum::Bool(true));
    static FALSE: ScalarCow = ScalarCow(ScalarCowEnum::Bool(false));

    #[test]
    fn test_to_str_bool() {
        assert_eq!(TRUE.to_str(), "true");
    }

    #[test]
    fn test_to_str_integer() {
        let val: ScalarCow = 42i32.into();
        assert_eq!(val.to_str(), "42");
    }

    #[test]
    fn test_to_str_float() {
        let val: ScalarCow = 42f64.into();
        assert_eq!(val.to_str(), "42");

        let val: ScalarCow = 42.34.into();
        assert_eq!(val.to_str(), "42.34");
    }

    #[test]
    fn test_to_str_str() {
        let val: ScalarCow = "foobar".into();
        assert_eq!(val.to_str(), "foobar");
    }

    #[test]
    fn test_to_integer_bool() {
        assert_eq!(TRUE.to_integer(), None);
    }

    #[test]
    fn test_to_integer_integer() {
        let val: ScalarCow = 42i32.into();
        assert_eq!(val.to_integer(), Some(42i32));
    }

    #[test]
    fn test_to_integer_float() {
        let val: ScalarCow = 42f64.into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow = 42.34.into();
        assert_eq!(val.to_integer(), None);
    }

    #[test]
    fn test_to_integer_str() {
        let val: ScalarCow = "foobar".into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow = "42.34".into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow = "42".into();
        assert_eq!(val.to_integer(), Some(42));
    }

    #[test]
    fn test_to_float_bool() {
        assert_eq!(TRUE.to_float(), None);
    }

    #[test]
    fn test_to_float_integer() {
        let val: ScalarCow = 42i32.into();
        assert_eq!(val.to_float(), Some(42f64));
    }

    #[test]
    fn test_to_float_float() {
        let val: ScalarCow = 42f64.into();
        assert_eq!(val.to_float(), Some(42f64));

        let val: ScalarCow = 42.34.into();
        assert_eq!(val.to_float(), Some(42.34));
    }

    #[test]
    fn test_to_float_str() {
        let val: ScalarCow = "foobar".into();
        assert_eq!(val.to_float(), None);

        let val: ScalarCow = "42.34".into();
        assert_eq!(val.to_float(), Some(42.34));

        let val: ScalarCow = "42".into();
        assert_eq!(val.to_float(), Some(42f64));
    }

    #[test]
    fn test_to_bool_bool() {
        assert_eq!(TRUE.to_bool(), Some(true));
    }

    #[test]
    fn test_to_bool_integer() {
        let val: ScalarCow = 42i32.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_float() {
        let val: ScalarCow = 42f64.into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow = 42.34.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_str() {
        let val: ScalarCow = "foobar".into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow = "true".into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow = "false".into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn integer_equality() {
        let val: ScalarCow = 42i32.into();
        let zero: ScalarCow = 0i32.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn integers_have_ruby_truthiness() {
        let val: ScalarCow = 42i32.into();
        let zero: ScalarCow = 0i32.into();
        assert_eq!(TRUE, val);
        assert_eq!(val, TRUE);
        assert!(val.is_truthy());

        assert_eq!(TRUE, zero);
        assert_eq!(zero, TRUE);
        assert!(zero.is_truthy());
    }

    #[test]
    fn float_equality() {
        let val: ScalarCow = 42f64.into();
        let zero: ScalarCow = 0f64.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn floats_have_ruby_truthiness() {
        let val: ScalarCow = 42f64.into();
        let zero: ScalarCow = 0f64.into();
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
        let alpha: ScalarCow = "alpha".into();
        let beta: ScalarCow = "beta".into();
        let empty: ScalarCow = "".into();
        assert_eq!(alpha, alpha);
        assert_eq!(empty, empty);
        assert!(alpha != beta);
        assert!(beta != alpha);
    }

    #[test]
    fn strings_have_ruby_truthiness() {
        // all strings in ruby are true
        let alpha: ScalarCow = "alpha".into();
        let empty: ScalarCow = "".into();
        assert_eq!(TRUE, alpha);
        assert_eq!(alpha, TRUE);
        assert!(alpha.is_truthy());

        assert_eq!(TRUE, empty);
        assert_eq!(empty, TRUE);
        assert!(empty.is_truthy());
    }

    #[test]
    fn parse_date_empty_is_bad() {
        assert!(parse_date("").is_none());
    }

    #[test]
    fn parse_date_bad() {
        assert!(parse_date("aaaaa").is_none());
    }

    #[test]
    fn parse_date_now() {
        assert!(parse_date("now").is_some());
    }

    #[test]
    fn parse_date_today() {
        assert!(parse_date("today").is_some());
    }
}
