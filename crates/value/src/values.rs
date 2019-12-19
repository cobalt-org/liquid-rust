use std::cmp::Ordering;
use std::fmt;

use sstring::SStringCow;

use crate::map;
use crate::Scalar;
use crate::ScalarCow;

/// An enum to represent different value types
pub type Value = ValueCow<'static>;

/// An enum to represent different value types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueCow<'v> {
    /// A scalar value.
    Scalar(ScalarCow<'v>),
    /// A sequence of `Value`s.
    Array(Array),
    /// A sequence of key/`Value` pairs.
    Object(Object),
    /// Nothing.
    Nil,
    /// No content.
    Empty,
    /// Evaluates to empty string.
    Blank,
}

/// Type representing a Liquid array, payload of the `Value::Array` variant
pub type Array = Vec<Value>;

/// Type representing a Liquid object, payload of the `Value::Object` variant
pub type Object = map::Map;

impl<'v> ValueCow<'v> {
    /// Create as a `Scalar`.
    pub fn scalar<T: Into<ScalarCow<'v>>>(value: T) -> Self {
        ValueCow::Scalar(ScalarCow::new(value))
    }

    /// Create as an `Array`.
    pub fn array<I: IntoIterator<Item = Value>>(iter: I) -> Value {
        let v: Array = iter.into_iter().collect();
        ValueCow::Array(v)
    }

    /// A `Display` for a `Scalar` as source code.
    pub fn source(&self) -> ValueSource<'_> {
        ValueSource(&self)
    }

    /// A `Display` for a `Value` rendered for the user.
    pub fn render(&self) -> ValueRendered<'_> {
        ValueRendered(&self)
    }

    /// Interpret as a string.
    pub fn to_sstr(&self) -> SStringCow<'_> {
        match self {
            ValueCow::Scalar(x) => x.to_sstr(),
            ValueCow::Array(_) | ValueCow::Object(_) => self.render().to_string().into(),
            ValueCow::Nil | ValueCow::Empty | ValueCow::Blank => SStringCow::default(),
        }
    }

    /// Extracts the scalar value if it is a scalar.
    pub fn as_scalar(&self) -> Option<&ScalarCow<'v>> {
        match self {
            ValueCow::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the scalar value if it is a scalar.
    pub fn into_scalar(self) -> Option<ScalarCow<'v>> {
        match self {
            ValueCow::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is a scalar
    pub fn is_scalar(&self) -> bool {
        self.as_scalar().is_some()
    }

    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            ValueCow::Array(ref s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the array value if it is an array.
    pub fn into_array(self) -> Option<Array> {
        match self {
            ValueCow::Array(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is an array
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Extracts the object value if it is a object.
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            ValueCow::Object(ref s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the object value if it is a object.
    pub fn into_object(self) -> Option<Object> {
        match self {
            ValueCow::Object(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is an object
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// Tests whether this value is Nil
    pub fn is_nil(&self) -> bool {
        match self {
            ValueCow::Nil => true,
            _ => false,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_truthy(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match self {
            ValueCow::Scalar(ref x) => x.is_truthy(),
            ValueCow::Nil | ValueCow::Empty | ValueCow::Blank => false,
            ValueCow::Array(_) | ValueCow::Object(_) => true,
        }
    }

    /// Whether a default constructed value.
    pub fn is_default(&self) -> bool {
        match self {
            ValueCow::Scalar(ref x) => x.is_default(),
            ValueCow::Nil => true,
            ValueCow::Empty => true,
            ValueCow::Blank => true,
            ValueCow::Array(ref x) => x.is_empty(),
            ValueCow::Object(ref x) => x.is_empty(),
        }
    }

    /// Tests whether this value is empty
    pub fn is_empty(&self) -> bool {
        // encode a best-guess of empty rules
        // See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        match self {
            ValueCow::Scalar(ref x) => x.is_empty(),
            ValueCow::Nil => true,
            ValueCow::Empty => true,
            ValueCow::Blank => true,
            ValueCow::Array(ref x) => x.is_empty(),
            ValueCow::Object(ref x) => x.is_empty(),
        }
    }

    /// Tests whether this value is blank
    pub fn is_blank(&self) -> bool {
        // encode a best-guess of empty rules
        // See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        match self {
            ValueCow::Scalar(ref x) => x.is_blank(),
            ValueCow::Nil => true,
            ValueCow::Empty => true,
            ValueCow::Blank => true,
            ValueCow::Array(ref x) => x.is_empty(),
            ValueCow::Object(ref x) => x.is_empty(),
        }
    }

    /// Report the data type (generally for error reporting).
    pub fn type_name(&self) -> &'static str {
        match self {
            ValueCow::Scalar(ref x) => x.type_name(),
            ValueCow::Nil => "nil",
            ValueCow::Empty => "empty",
            ValueCow::Blank => "blank",
            ValueCow::Array(_) => "array",
            ValueCow::Object(_) => "object",
        }
    }

    /// Access a contained `Value`.
    pub fn contains_key(&self, index: &ScalarCow<'_>) -> bool {
        match self {
            ValueCow::Array(ref x) => {
                if let Some(index) = index.to_integer() {
                    let index = convert_index(index, x.len());
                    index < x.len()
                } else {
                    match &*index.to_sstr() {
                        "first" | "last" => true,
                        _ => false,
                    }
                }
            }
            ValueCow::Object(ref x) => x.contains_key(index.to_sstr().as_str()),
            _ => false,
        }
    }

    /// Keys available for lookup.
    pub fn keys(&self) -> Keys<'_> {
        let v = match self {
            ValueCow::Array(ref x) => {
                let start: i32 = 0;
                let end = x.len() as i32;
                let keys: Vec<_> = (start..end).map(Scalar::new).collect();
                keys
            }
            ValueCow::Object(x) => x.keys().map(|s| ScalarCow::new(s)).collect(),
            _ => vec![],
        };
        Keys(v.into_iter())
    }

    /// Access a contained `Value`.
    pub fn get<'s>(&'s self, index: &ScalarCow<'_>) -> Option<&'s Self> {
        match self {
            ValueCow::Array(ref x) => {
                if let Some(index) = index.to_integer() {
                    let index = convert_index(index, x.len());
                    x.get(index as usize)
                } else {
                    match &*index.to_sstr() {
                        "first" => x.get(0),
                        "last" => x.get(x.len() - 1),
                        _ => None,
                    }
                }
            }
            ValueCow::Object(ref x) => x.get(index.to_sstr().as_str()),
            _ => None,
        }
    }
}

/// Iterator over a `Value`s keys.
#[derive(Debug)]
pub struct Keys<'s>(::std::vec::IntoIter<ScalarCow<'s>>);

impl<'s> Iterator for Keys<'s> {
    type Item = ScalarCow<'s>;

    #[inline]
    fn next(&mut self) -> Option<ScalarCow<'s>> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }
}

impl<'s> ExactSizeIterator for Keys<'s> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

fn convert_index(index: i32, max_size: usize) -> usize {
    let index = index as isize;
    let max_size = max_size as isize;
    let index = if 0 <= index { index } else { max_size + index };
    index as usize
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Self) -> bool {
        value_eq(self, other)
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl PartialEq<crate::DateTime> for Value {
    fn eq(&self, other: &crate::DateTime) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl PartialEq<crate::Date> for Value {
    fn eq(&self, other: &crate::Date) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        value_eq(self, &ValueCow::scalar(other))
    }
}

impl<'s> PartialEq<&'s str> for Value {
    fn eq(&self, other: &&str) -> bool {
        value_eq(self, &ValueCow::scalar(*other))
    }
}

impl<'s> PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        value_eq(self, &ValueCow::scalar(other.as_str()))
    }
}

impl PartialEq<sstring::SString> for Value {
    fn eq(&self, other: &sstring::SString) -> bool {
        value_eq(self, &ValueCow::scalar(other))
    }
}

impl<'s> PartialEq<sstring::SStringRef<'s>> for Value {
    fn eq(&self, other: &sstring::SStringRef<'s>) -> bool {
        value_eq(self, &ValueCow::scalar(other))
    }
}

impl<'s> PartialEq<sstring::SStringCow<'s>> for Value {
    fn eq(&self, other: &sstring::SStringCow<'s>) -> bool {
        value_eq(self, &ValueCow::scalar(other))
    }
}

impl Eq for Value {}

impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        value_cmp(self, other)
    }
}

/// A `Display` for a `Scalar` as source code.
#[derive(Debug)]
pub struct ValueSource<'s>(&'s ValueCow<'s>);

impl<'s> fmt::Display for ValueSource<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ValueCow::Scalar(ref x) => write!(f, "{}", x.render())?,
            ValueCow::Array(ref x) => {
                write!(f, "[")?;
                for item in x {
                    write!(f, "{}, ", item.render())?;
                }
                write!(f, "]")?;
            }
            ValueCow::Object(ref x) => {
                write!(f, "{{")?;
                for (k, v) in x {
                    write!(f, r#""{}": {}, "#, k, v.render())?;
                }
                write!(f, "}}")?;
            }
            ValueCow::Nil => write!(f, "nil")?,
            ValueCow::Empty => write!(f, "empty")?,
            ValueCow::Blank => write!(f, "blank")?,
        }
        Ok(())
    }
}

/// A `Display` for a `Value` rendered for the user.
#[derive(Debug)]
pub struct ValueRendered<'s>(&'s ValueCow<'s>);

impl<'s> fmt::Display for ValueRendered<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Must match `ValueCow::to_str`
        match self.0 {
            ValueCow::Scalar(ref x) => write!(f, "{}", x.render())?,
            ValueCow::Array(ref x) => {
                for item in x {
                    write!(f, "{}", item.render())?;
                }
            }
            ValueCow::Object(ref x) => {
                for (k, v) in x {
                    write!(f, "{}{}", k, v.render())?;
                }
            }
            ValueCow::Nil | ValueCow::Empty | ValueCow::Blank => (),
        }
        Ok(())
    }
}

fn value_eq<'v>(lhs: &'v ValueCow<'v>, rhs: &'v ValueCow<'v>) -> bool {
    match (lhs, rhs) {
        (&ValueCow::Scalar(ref x), &ValueCow::Scalar(ref y)) => x == y,
        (&ValueCow::Array(ref x), &ValueCow::Array(ref y)) => x == y,
        (&ValueCow::Object(ref x), &ValueCow::Object(ref y)) => x == y,

        // encode a best-guess of empty rules
        // See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        (&ValueCow::Nil, &ValueCow::Nil)
        | (&ValueCow::Empty, &ValueCow::Empty)
        | (&ValueCow::Blank, &ValueCow::Blank)
        | (&ValueCow::Empty, &ValueCow::Blank)
        | (&ValueCow::Blank, &ValueCow::Empty)
        | (&ValueCow::Nil, &ValueCow::Blank)
        | (&ValueCow::Blank, &ValueCow::Nil) => true,

        (&ValueCow::Empty, s) | (s, &ValueCow::Empty) => s.is_empty(),

        (&ValueCow::Blank, s) | (s, &ValueCow::Blank) => s.is_blank(),

        // encode Ruby truthiness: all values except false and nil are true
        (&ValueCow::Nil, &ValueCow::Scalar(ref b)) | (&ValueCow::Scalar(ref b), &ValueCow::Nil) => {
            !b.to_bool().unwrap_or(true)
        }
        (_, &ValueCow::Scalar(ref b)) | (&ValueCow::Scalar(ref b), _) => {
            b.to_bool().unwrap_or(false)
        }

        _ => false,
    }
}

fn value_cmp(lhs: &ValueCow, rhs: &ValueCow) -> Option<Ordering> {
    match (lhs, rhs) {
        (&ValueCow::Scalar(ref x), &ValueCow::Scalar(ref y)) => x.partial_cmp(y),
        (&ValueCow::Array(ref x), &ValueCow::Array(ref y)) => x.iter().partial_cmp(y.iter()),
        (&ValueCow::Object(ref x), &ValueCow::Object(ref y)) => x.iter().partial_cmp(y.iter()),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_string_scalar() {
        let val = ValueCow::scalar(42f64);
        assert_eq!(&val.render().to_string(), "42");
        assert_eq!(&val.to_sstr(), "42");
    }

    #[test]
    fn test_to_string_array() {
        let val = ValueCow::Array(vec![
            ValueCow::scalar(3f64),
            ValueCow::scalar("test"),
            ValueCow::scalar(5.3),
        ]);
        assert_eq!(&val.render().to_string(), "3test5.3");
        assert_eq!(&val.to_sstr(), "3test5.3");
    }

    // TODO make a test for object, remember values are in arbitrary orders in HashMaps

    #[test]
    fn test_to_string_nil() {
        assert_eq!(&ValueCow::Nil.render().to_string(), "");
        assert_eq!(&ValueCow::Nil.to_sstr(), "");
    }

    #[test]
    fn scalar_equality() {
        assert_eq!(ValueCow::scalar("alpha"), ValueCow::scalar("alpha"));
        assert_eq!(ValueCow::scalar(""), ValueCow::scalar(""));
        assert!(ValueCow::scalar("alpha") != ValueCow::scalar("beta"));
        assert!(ValueCow::scalar("beta") != ValueCow::scalar("alpha"));
    }

    #[test]
    fn scalars_have_ruby_truthiness() {
        // all strings in ruby are true
        assert_eq!(
            ValueCow::scalar(true),
            ValueCow::scalar("All strings are truthy")
        );
        assert_eq!(ValueCow::scalar(true), ValueCow::scalar(""));
        assert!(ValueCow::scalar("").is_truthy());

        assert_eq!(ValueCow::scalar(true), ValueCow::scalar(true));
        assert!(ValueCow::scalar(true) != ValueCow::scalar(false));
    }

    #[test]
    fn array_equality() {
        let a = ValueCow::Array(vec![ValueCow::scalar("one"), ValueCow::scalar("two")]);
        let b = ValueCow::Array(vec![ValueCow::scalar("alpha"), ValueCow::scalar("beta")]);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn arrays_have_ruby_truthiness() {
        assert_eq!(ValueCow::scalar(true), ValueCow::Array(Vec::new()));
        assert!(ValueCow::Array(Vec::new()).is_truthy());
    }

    #[test]
    fn object_equality() {
        let a: Object = [
            ("alpha".into(), ValueCow::scalar("1")),
            ("beta".into(), ValueCow::scalar(2f64)),
        ]
        .iter()
        .cloned()
        .collect();
        let a = ValueCow::Object(a);

        let b: Object = [
            ("alpha".into(), ValueCow::scalar("1")),
            ("beta".into(), ValueCow::scalar(2f64)),
            ("gamma".into(), ValueCow::Array(vec![])),
        ]
        .iter()
        .cloned()
        .collect();
        let b = ValueCow::Object(b);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn objects_have_ruby_truthiness() {
        assert_eq!(ValueCow::scalar(true), ValueCow::Object(Object::new()));
        assert!(ValueCow::Object(Object::new()).is_truthy());
    }

    #[test]
    fn nil_equality() {
        assert_eq!(ValueCow::Nil, ValueCow::Nil);
    }

    #[test]
    fn nils_have_ruby_truthiness() {
        assert_eq!(ValueCow::scalar(false), ValueCow::Nil);
        assert!(!ValueCow::Nil.is_truthy());

        assert_eq!(ValueCow::scalar(false), ValueCow::Nil);
        assert!(ValueCow::scalar(true) != ValueCow::Nil);
        assert!(ValueCow::scalar("") != ValueCow::Nil);
    }

    #[test]
    fn empty_equality() {
        // Truth table from https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        assert_eq!(ValueCow::Empty, ValueCow::Empty);
        assert_eq!(ValueCow::Empty, ValueCow::Blank);
        assert_eq!(ValueCow::Empty, liquid_value!(""));
        assert_ne!(ValueCow::Empty, liquid_value!(" "));
        assert_eq!(ValueCow::Empty, liquid_value!([]));
        assert_ne!(ValueCow::Empty, liquid_value!([nil]));
        assert_eq!(ValueCow::Empty, liquid_value!({}));
        assert_ne!(ValueCow::Empty, liquid_value!({ "a": nil }));
    }

    #[test]
    fn blank_equality() {
        // Truth table from https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        assert_eq!(ValueCow::Blank, ValueCow::Blank);
        assert_eq!(ValueCow::Blank, ValueCow::Empty);
        assert_eq!(ValueCow::Blank, liquid_value!(nil));
        assert_eq!(ValueCow::Blank, liquid_value!(false));
        assert_ne!(ValueCow::Blank, liquid_value!(true));
        assert_ne!(ValueCow::Blank, liquid_value!(0));
        assert_ne!(ValueCow::Blank, liquid_value!(1));
        assert_eq!(ValueCow::Blank, liquid_value!(""));
        assert_eq!(ValueCow::Blank, liquid_value!(" "));
        assert_eq!(ValueCow::Blank, liquid_value!([]));
        assert_ne!(ValueCow::Blank, liquid_value!([nil]));
        assert_eq!(ValueCow::Blank, liquid_value!({}));
        assert_ne!(ValueCow::Blank, liquid_value!({ "a": nil }));
    }
}
