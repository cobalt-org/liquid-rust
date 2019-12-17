use std::cmp::Ordering;
use std::fmt;

use sstring::SString;
use sstring::SStringCow;
use sstring::SStringRef;

use crate::{Date, DateTime};

/// A Liquid scalar value
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
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
    DateTime(DateTime),
    Date(Date),
    Str(SStringCow<'s>),
}

impl<'s> ScalarCow<'s> {
    /// Convert a value into a `ScalarCow`.
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }

    /// A `Display` for a `Scalar` as source code.
    pub fn source(&self) -> ScalarSource<'_> {
        ScalarSource(&self.0)
    }

    /// A `Display` for a `Scalar` rendered for the user.
    pub fn render(&self) -> ScalarRendered<'_> {
        ScalarRendered(&self.0)
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
            ScalarCowEnum::Integer(x) => ScalarCow::new(x),
            ScalarCowEnum::Float(x) => ScalarCow::new(x),
            ScalarCowEnum::Bool(x) => ScalarCow::new(x),
            ScalarCowEnum::DateTime(x) => ScalarCow::new(x),
            ScalarCowEnum::Date(x) => ScalarCow::new(x),
            ScalarCowEnum::Str(ref x) => ScalarCow::new(x.as_ref()),
        }
    }

    /// Interpret as a string.
    pub fn to_sstr(&self) -> SStringCow<'_> {
        match self.0 {
            ScalarCowEnum::Integer(_)
            | ScalarCowEnum::Float(_)
            | ScalarCowEnum::Bool(_)
            | ScalarCowEnum::DateTime(_)
            | ScalarCowEnum::Date(_) => self.render().to_string().into(),
            ScalarCowEnum::Str(ref x) => x.as_ref().into(),
        }
    }

    /// Convert to a string.
    pub fn into_string(self) -> SString {
        match self.0 {
            ScalarCowEnum::Integer(x) => x.to_string().into(),
            ScalarCowEnum::Float(x) => x.to_string().into(),
            ScalarCowEnum::Bool(x) => x.to_string().into(),
            ScalarCowEnum::DateTime(x) => x.to_string().into(),
            ScalarCowEnum::Date(x) => x.to_string().into(),
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

    /// Interpret as a date time, if possible
    pub fn to_date_time(&self) -> Option<DateTime> {
        match self.0 {
            ScalarCowEnum::DateTime(ref x) => Some(*x),
            ScalarCowEnum::Str(ref x) => DateTime::from_str(x.as_str()),
            _ => None,
        }
    }

    /// Interpret as a date time, if possible
    pub fn to_date(&self) -> Option<Date> {
        match self.0 {
            ScalarCowEnum::DateTime(ref x) => Some(x.date()),
            ScalarCowEnum::Date(ref x) => Some(*x),
            ScalarCowEnum::Str(ref x) => Date::from_str(x.as_str()),
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
            ScalarCowEnum::DateTime(_) => "date time",
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

impl<'s> From<DateTime> for ScalarCow<'s> {
    fn from(s: DateTime) -> Self {
        ScalarCow {
            0: ScalarCowEnum::DateTime(s),
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

impl<'s> From<SString> for ScalarCow<'s> {
    fn from(s: SString) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.into()),
        }
    }
}

impl<'s> From<&'s SString> for ScalarCow<'s> {
    fn from(s: &'s SString) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.as_ref().into()),
        }
    }
}

impl<'s> From<SStringRef<'s>> for ScalarCow<'s> {
    fn from(s: SStringRef<'s>) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.into()),
        }
    }
}

impl<'s> From<&'s SStringRef<'s>> for ScalarCow<'s> {
    fn from(s: &'s SStringRef<'s>) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.into()),
        }
    }
}

impl<'s> From<SStringCow<'s>> for ScalarCow<'s> {
    fn from(s: SStringCow<'s>) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s),
        }
    }
}

impl<'s> From<&'s SStringCow<'s>> for ScalarCow<'s> {
    fn from(s: &'s SStringCow<'s>) -> Self {
        ScalarCow {
            0: ScalarCowEnum::Str(s.clone()),
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

impl<'s> PartialEq<ScalarCow<'s>> for ScalarCow<'s> {
    fn eq(&self, other: &Self) -> bool {
        scalar_eq(self, other)
    }
}

impl<'s> PartialEq<i32> for ScalarCow<'s> {
    fn eq(&self, other: &i32) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<f64> for ScalarCow<'s> {
    fn eq(&self, other: &f64) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<bool> for ScalarCow<'s> {
    fn eq(&self, other: &bool) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<DateTime> for ScalarCow<'s> {
    fn eq(&self, other: &DateTime) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<Date> for ScalarCow<'s> {
    fn eq(&self, other: &Date) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<str> for ScalarCow<'s> {
    fn eq(&self, other: &str) -> bool {
        let other = other.into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<&'s str> for ScalarCow<'s> {
    fn eq(&self, other: &&str) -> bool {
        let other = (*other).into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<String> for ScalarCow<'s> {
    fn eq(&self, other: &String) -> bool {
        let other = other.into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<SString> for ScalarCow<'s> {
    fn eq(&self, other: &SString) -> bool {
        let other = other.into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<SStringRef<'s>> for ScalarCow<'s> {
    fn eq(&self, other: &SStringRef<'s>) -> bool {
        let other = other.into();
        scalar_eq(self, &other)
    }
}

impl<'s> PartialEq<SStringCow<'s>> for ScalarCow<'s> {
    fn eq(&self, other: &SStringCow<'s>) -> bool {
        let other = other.into();
        scalar_eq(self, &other)
    }
}

impl<'s> Eq for ScalarCow<'s> {}

impl<'s> PartialOrd<ScalarCow<'s>> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        scalar_cmp(self, other)
    }
}

impl<'s> PartialOrd<i32> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        let other = (*other).into();
        scalar_cmp(self, &other)
    }
}

impl<'s> PartialOrd<f64> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        let other = (*other).into();
        scalar_cmp(self, &other)
    }
}

impl<'s> PartialOrd<bool> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        let other = (*other).into();
        scalar_cmp(self, &other)
    }
}

impl<'s> PartialOrd<DateTime> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &DateTime) -> Option<Ordering> {
        let other = (*other).into();
        scalar_cmp(self, &other)
    }
}

impl<'s> PartialOrd<Date> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        let other = (*other).into();
        scalar_cmp(self, &other)
    }
}

impl<'s> PartialOrd<str> for ScalarCow<'s> {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        let other = other.into();
        scalar_cmp(self, &other)
    }
}

/// A `Display` for a `Scalar` as source code.
#[derive(Debug)]
pub struct ScalarSource<'s>(&'s ScalarCowEnum<'s>);

impl<'s> fmt::Display for ScalarSource<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ScalarCowEnum::Integer(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Float(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Bool(ref x) => write!(f, "{}", x),
            ScalarCowEnum::DateTime(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Date(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Str(ref x) => write!(f, r#""{}""#, x),
        }
    }
}

/// A `Display` for a `Scalar` rendered for the user.
#[derive(Debug)]
pub struct ScalarRendered<'s>(&'s ScalarCowEnum<'s>);

impl<'s> fmt::Display for ScalarRendered<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Must match `ScalarCow::to_str`
        match self.0 {
            ScalarCowEnum::Integer(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Float(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Bool(ref x) => write!(f, "{}", x),
            ScalarCowEnum::DateTime(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Date(ref x) => write!(f, "{}", x),
            ScalarCowEnum::Str(ref x) => write!(f, "{}", x),
        }
    }
}

fn scalar_eq<'s>(lhs: &ScalarCow<'s>, rhs: &ScalarCow<'s>) -> bool {
    match (&lhs.0, &rhs.0) {
        (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Integer(y)) => x == y,
        (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Float(y)) => (f64::from(x)) == y,
        (&ScalarCowEnum::Float(x), &ScalarCowEnum::Integer(y)) => x == (f64::from(y)),
        (&ScalarCowEnum::Float(x), &ScalarCowEnum::Float(y)) => x == y,
        (&ScalarCowEnum::Bool(x), &ScalarCowEnum::Bool(y)) => x == y,
        (&ScalarCowEnum::DateTime(x), &ScalarCowEnum::DateTime(y)) => x == y,
        (&ScalarCowEnum::Date(x), &ScalarCowEnum::Date(y)) => x == y,
        (&ScalarCowEnum::DateTime(x), &ScalarCowEnum::Date(y)) => x == x.with_date(y),
        (&ScalarCowEnum::Date(x), &ScalarCowEnum::DateTime(y)) => y.with_date(x) == y,
        (&ScalarCowEnum::Str(ref x), &ScalarCowEnum::Str(ref y)) => x == y,
        // encode Ruby truthiness: all values except false and nil are true
        (_, &ScalarCowEnum::Bool(b)) | (&ScalarCowEnum::Bool(b), _) => b,
        _ => false,
    }
}

fn scalar_cmp<'s>(lhs: &ScalarCow<'s>, rhs: &ScalarCow<'s>) -> Option<Ordering> {
    match (&lhs.0, &rhs.0) {
        (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Integer(y)) => x.partial_cmp(&y),
        (&ScalarCowEnum::Integer(x), &ScalarCowEnum::Float(y)) => (f64::from(x)).partial_cmp(&y),
        (&ScalarCowEnum::Float(x), &ScalarCowEnum::Integer(y)) => x.partial_cmp(&(f64::from(y))),
        (&ScalarCowEnum::Float(x), &ScalarCowEnum::Float(y)) => x.partial_cmp(&y),
        (&ScalarCowEnum::Bool(x), &ScalarCowEnum::Bool(y)) => x.partial_cmp(&y),
        (&ScalarCowEnum::DateTime(x), &ScalarCowEnum::DateTime(y)) => x.partial_cmp(&y),
        (&ScalarCowEnum::Date(x), &ScalarCowEnum::Date(y)) => x.partial_cmp(&y),
        (&ScalarCowEnum::DateTime(x), &ScalarCowEnum::Date(y)) => x.partial_cmp(&x.with_date(y)),
        (&ScalarCowEnum::Date(x), &ScalarCowEnum::DateTime(y)) => y.with_date(x).partial_cmp(&y),
        (&ScalarCowEnum::Str(ref x), &ScalarCowEnum::Str(ref y)) => x.partial_cmp(y),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static TRUE: ScalarCow<'_> = ScalarCow(ScalarCowEnum::Bool(true));
    static FALSE: ScalarCow<'_> = ScalarCow(ScalarCowEnum::Bool(false));

    #[test]
    fn test_to_str_bool() {
        assert_eq!(TRUE.to_sstr(), "true");
    }

    #[test]
    fn test_to_str_integer() {
        let val: ScalarCow<'_> = 42i32.into();
        assert_eq!(val.to_sstr(), "42");
    }

    #[test]
    fn test_to_str_float() {
        let val: ScalarCow<'_> = 42f64.into();
        assert_eq!(val.to_sstr(), "42");

        let val: ScalarCow<'_> = 42.34.into();
        assert_eq!(val.to_sstr(), "42.34");
    }

    #[test]
    fn test_to_str_str() {
        let val: ScalarCow<'_> = "foobar".into();
        assert_eq!(val.to_sstr(), "foobar");
    }

    #[test]
    fn test_to_integer_bool() {
        assert_eq!(TRUE.to_integer(), None);
    }

    #[test]
    fn test_to_integer_integer() {
        let val: ScalarCow<'_> = 42i32.into();
        assert_eq!(val.to_integer(), Some(42i32));
    }

    #[test]
    fn test_to_integer_float() {
        let val: ScalarCow<'_> = 42f64.into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow<'_> = 42.34.into();
        assert_eq!(val.to_integer(), None);
    }

    #[test]
    fn test_to_integer_str() {
        let val: ScalarCow<'_> = "foobar".into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow<'_> = "42.34".into();
        assert_eq!(val.to_integer(), None);

        let val: ScalarCow<'_> = "42".into();
        assert_eq!(val.to_integer(), Some(42));
    }

    #[test]
    fn test_to_float_bool() {
        assert_eq!(TRUE.to_float(), None);
    }

    #[test]
    fn test_to_float_integer() {
        let val: ScalarCow<'_> = 42i32.into();
        assert_eq!(val.to_float(), Some(42f64));
    }

    #[test]
    fn test_to_float_float() {
        let val: ScalarCow<'_> = 42f64.into();
        assert_eq!(val.to_float(), Some(42f64));

        let val: ScalarCow<'_> = 42.34.into();
        assert_eq!(val.to_float(), Some(42.34));
    }

    #[test]
    fn test_to_float_str() {
        let val: ScalarCow<'_> = "foobar".into();
        assert_eq!(val.to_float(), None);

        let val: ScalarCow<'_> = "42.34".into();
        assert_eq!(val.to_float(), Some(42.34));

        let val: ScalarCow<'_> = "42".into();
        assert_eq!(val.to_float(), Some(42f64));
    }

    #[test]
    fn test_to_bool_bool() {
        assert_eq!(TRUE.to_bool(), Some(true));
    }

    #[test]
    fn test_to_bool_integer() {
        let val: ScalarCow<'_> = 42i32.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_float() {
        let val: ScalarCow<'_> = 42f64.into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow<'_> = 42.34.into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn test_to_bool_str() {
        let val: ScalarCow<'_> = "foobar".into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow<'_> = "true".into();
        assert_eq!(val.to_bool(), None);

        let val: ScalarCow<'_> = "false".into();
        assert_eq!(val.to_bool(), None);
    }

    #[test]
    fn integer_equality() {
        let val: ScalarCow<'_> = 42i32.into();
        let zero: ScalarCow<'_> = 0i32.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn integers_have_ruby_truthiness() {
        let val: ScalarCow<'_> = 42i32.into();
        let zero: ScalarCow<'_> = 0i32.into();
        assert_eq!(TRUE, val);
        assert_eq!(val, TRUE);
        assert!(val.is_truthy());

        assert_eq!(TRUE, zero);
        assert_eq!(zero, TRUE);
        assert!(zero.is_truthy());
    }

    #[test]
    fn float_equality() {
        let val: ScalarCow<'_> = 42f64.into();
        let zero: ScalarCow<'_> = 0f64.into();
        assert_eq!(val, val);
        assert_eq!(zero, zero);
        assert!(val != zero);
        assert!(zero != val);
    }

    #[test]
    fn floats_have_ruby_truthiness() {
        let val: ScalarCow<'_> = 42f64.into();
        let zero: ScalarCow<'_> = 0f64.into();
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
        let alpha: ScalarCow<'_> = "alpha".into();
        let beta: ScalarCow<'_> = "beta".into();
        let empty: ScalarCow<'_> = "".into();
        assert_eq!(alpha, alpha);
        assert_eq!(empty, empty);
        assert!(alpha != beta);
        assert!(beta != alpha);
    }

    #[test]
    fn strings_have_ruby_truthiness() {
        // all strings in ruby are true
        let alpha: ScalarCow<'_> = "alpha".into();
        let empty: ScalarCow<'_> = "".into();
        assert_eq!(TRUE, alpha);
        assert_eq!(alpha, TRUE);
        assert!(alpha.is_truthy());

        assert_eq!(TRUE, empty);
        assert_eq!(empty, TRUE);
        assert!(empty.is_truthy());
    }
}
