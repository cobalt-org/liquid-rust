use std::borrow;
use std::cmp::Ordering;
use std::fmt;

use itertools;

use super::map;
use super::Scalar;
use super::ScalarCow;

/// An enum to represent different value types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// A scalar value.
    Scalar(Scalar),
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

impl Value {
    /// Create as a `Scalar`.
    pub fn scalar<T: Into<Scalar>>(value: T) -> Self {
        Value::Scalar(Scalar::new(value))
    }

    /// Create as an `Array`.
    pub fn array<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let v: Array = iter.into_iter().collect();
        Value::Array(v)
    }

    /// Create as nothing.
    pub fn nil() -> Self {
        Value::Nil
    }

    /// A `Display` for a `Scalar` as source code.
    pub fn source(&self) -> ValueSource {
        ValueSource(&self)
    }

    /// A `Display` for a `Value` rendered for the user.
    pub fn render(&self) -> ValueRendered {
        ValueRendered(&self)
    }

    /// Interpret as a string.
    pub fn to_str(&self) -> borrow::Cow<str> {
        match *self {
            Value::Scalar(ref x) => x.to_str(),
            Value::Array(ref x) => {
                let arr: Vec<_> = x.iter().map(|v| v.render()).collect();
                borrow::Cow::Owned(itertools::join(arr, ""))
            }
            Value::Object(ref x) => {
                let arr: Vec<_> = x
                    .iter()
                    .map(|(k, v)| format!("{}{}", k, v.render()))
                    .collect();
                borrow::Cow::Owned(itertools::join(arr, ""))
            }
            Value::Nil | Value::Empty | Value::Blank => borrow::Cow::Borrowed(""),
        }
    }

    /// Extracts the scalar value if it is a scalar.
    pub fn as_scalar(&self) -> Option<&Scalar> {
        match *self {
            Value::Scalar(ref s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the scalar value if it is a scalar.
    pub fn into_scalar(self) -> Option<Scalar> {
        match self {
            Value::Scalar(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is a scalar
    pub fn is_scalar(&self) -> bool {
        self.as_scalar().is_some()
    }

    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Option<&Array> {
        match *self {
            Value::Array(ref s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the array value if it is an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match *self {
            Value::Array(ref mut s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the array value if it is an array.
    pub fn into_array(self) -> Option<Array> {
        match self {
            Value::Array(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is an array
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Extracts the object value if it is a object.
    pub fn as_object(&self) -> Option<&Object> {
        match *self {
            Value::Object(ref s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the object value if it is a object.
    pub fn as_object_mut(&mut self) -> Option<&mut Object> {
        match *self {
            Value::Object(ref mut s) => Some(s),
            _ => None,
        }
    }

    /// Extracts the object value if it is a object.
    pub fn into_object(self) -> Option<Object> {
        match self {
            Value::Object(s) => Some(s),
            _ => None,
        }
    }

    /// Tests whether this value is an object
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// Extracts the nil value if it is nil
    pub fn as_nil(&self) -> Option<()> {
        match *self {
            Value::Nil => Some(()),
            _ => None,
        }
    }

    /// Tests whether this value is nil
    pub fn is_nil(&self) -> bool {
        match *self {
            Value::Nil => true,
            _ => false,
        }
    }

    /// Extracts the empty value if it is empty
    pub fn as_empty(&self) -> Option<()> {
        match *self {
            Value::Empty => Some(()),
            _ => None,
        }
    }

    /// Tests whether this value is empty
    pub fn is_empty(&self) -> bool {
        match *self {
            Value::Empty => true,
            _ => false,
        }
    }

    /// Extracts the blank value if it is blank
    pub fn as_blank(&self) -> Option<()> {
        match *self {
            Value::Blank => Some(()),
            _ => None,
        }
    }

    /// Tests whether this value is blank
    pub fn is_blank(&self) -> bool {
        match *self {
            Value::Blank => true,
            _ => false,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_truthy(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match *self {
            Value::Scalar(ref x) => x.is_truthy(),
            Value::Nil | Value::Empty | Value::Blank => false,
            _ => true,
        }
    }

    /// Whether a default constructed value.
    pub fn is_default(&self) -> bool {
        match *self {
            Value::Scalar(ref x) => x.is_default(),
            Value::Nil => true,
            Value::Empty => true,
            Value::Blank => true,
            Value::Array(ref x) => x.is_empty(),
            Value::Object(ref x) => x.is_empty(),
        }
    }

    /// Report the data type (generally for error reporting).
    pub fn type_name(&self) -> &'static str {
        match *self {
            Value::Scalar(ref x) => x.type_name(),
            Value::Nil => "nil",
            Value::Empty => "empty",
            Value::Blank => "blank",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Access a contained `Value`.
    pub fn contains_key(&self, index: &Scalar) -> bool {
        match *self {
            Value::Array(ref x) => {
                if let Some(index) = index.to_integer() {
                    let index = convert_index(index, x.len());
                    index < x.len()
                } else {
                    match &*index.to_str() {
                        "first" | "last" => true,
                        _ => false,
                    }
                }
            }
            Value::Object(ref x) => x.contains_key(index.to_str().as_ref()),
            _ => false,
        }
    }

    /// Keys available for lookup.
    pub fn keys(&self) -> Keys {
        let v = match *self {
            Value::Array(ref x) => {
                let start: i32 = 0;
                let end = x.len() as i32;
                let mut keys: Vec<_> = (start..end).map(Scalar::new).collect();
                keys.push(Scalar::new("first"));
                keys.push(Scalar::new("last"));
                keys
            }
            Value::Object(ref x) => x
                .keys()
                .map(|s| match *s {
                    borrow::Cow::Borrowed(s) => Scalar::new(s),
                    borrow::Cow::Owned(ref s) => Scalar::new(s.to_owned()),
                })
                .collect(),
            _ => vec![],
        };
        Keys(v.into_iter())
    }

    /// Access a contained `Value`.
    pub fn get<'s>(&'s self, index: &ScalarCow) -> Option<&'s Self> {
        match *self {
            Value::Array(ref x) => {
                if let Some(index) = index.to_integer() {
                    let index = convert_index(index, x.len());
                    x.get(index as usize)
                } else {
                    match &*index.to_str() {
                        "first" => x.get(0),
                        "last" => x.get(x.len() - 1),
                        _ => None,
                    }
                }
            }
            Value::Object(ref x) => x.get(index.to_str().as_ref()),
            _ => None,
        }
    }
}

/// Iterator over a `Value`s keys.
#[derive(Debug)]
pub struct Keys(::std::vec::IntoIter<Scalar>);

impl Iterator for Keys {
    type Item = Scalar;

    #[inline]
    fn next(&mut self) -> Option<Scalar> {
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

impl ExactSizeIterator for Keys {
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
        Self::nil()
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Self) -> bool {
        value_eq(self, other)
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
pub struct ValueSource<'s>(&'s Value);

impl<'s> fmt::Display for ValueSource<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Value::Scalar(ref x) => write!(f, "{}", x.render())?,
            Value::Array(ref x) => {
                write!(f, "[")?;
                for item in x {
                    write!(f, "{}, ", item.render())?;
                }
                write!(f, "]")?;
            }
            Value::Object(ref x) => {
                write!(f, "{{")?;
                for (k, v) in x {
                    write!(f, r#""{}": {}, "#, k, v.render())?;
                }
                write!(f, "}}")?;
            }
            Value::Nil => write!(f, "nil")?,
            Value::Empty => write!(f, "empty")?,
            Value::Blank => write!(f, "blank")?,
        }
        Ok(())
    }
}

/// A `Display` for a `Value` rendered for the user.
#[derive(Debug)]
pub struct ValueRendered<'s>(&'s Value);

impl<'s> fmt::Display for ValueRendered<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Must match `Value::to_str`
        match self.0 {
            Value::Scalar(ref x) => write!(f, "{}", x.render())?,
            Value::Array(ref x) => {
                for item in x {
                    write!(f, "{}", item.render())?;
                }
            }
            Value::Object(ref x) => {
                for (k, v) in x {
                    write!(f, "{}{}", k, v.render())?;
                }
            }
            Value::Nil | Value::Empty | Value::Blank => (),
        }
        Ok(())
    }
}

fn value_eq(lhs: &Value, rhs: &Value) -> bool {
    match (lhs, rhs) {
        (&Value::Scalar(ref x), &Value::Scalar(ref y)) => x == y,
        (&Value::Array(ref x), &Value::Array(ref y)) => x == y,
        (&Value::Object(ref x), &Value::Object(ref y)) => x == y,
        (&Value::Nil, &Value::Nil)
        | (&Value::Empty, &Value::Empty)
        | (&Value::Blank, &Value::Blank)
        | (&Value::Empty, &Value::Blank)
        | (&Value::Blank, &Value::Empty) => true,

        // encode a best-guess of empty rules
        // See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        (&Value::Empty, &Value::Scalar(ref s)) | (&Value::Scalar(ref s), &Value::Empty) => {
            s.to_str().is_empty()
        }
        (&Value::Empty, &Value::Array(ref s)) | (&Value::Array(ref s), &Value::Empty) => {
            s.is_empty()
        }
        (&Value::Empty, &Value::Object(ref s)) | (&Value::Object(ref s), &Value::Empty) => {
            s.is_empty()
        }

        // encode a best-guess of blank rules
        // See tables in https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        (&Value::Nil, &Value::Blank) | (&Value::Blank, &Value::Nil) => true,
        (&Value::Blank, &Value::Scalar(ref s)) | (&Value::Scalar(ref s), &Value::Blank) => {
            s.to_str().trim().is_empty() || !s.to_bool().unwrap_or(true)
        }
        (&Value::Blank, &Value::Array(ref s)) | (&Value::Array(ref s), &Value::Blank) => {
            s.is_empty()
        }
        (&Value::Blank, &Value::Object(ref s)) | (&Value::Object(ref s), &Value::Blank) => {
            s.is_empty()
        }

        // encode Ruby truthiness: all values except false and nil are true
        (&Value::Nil, &Value::Scalar(ref b)) | (&Value::Scalar(ref b), &Value::Nil) => {
            !b.to_bool().unwrap_or(true)
        }
        (_, &Value::Scalar(ref b)) | (&Value::Scalar(ref b), _) => b.to_bool().unwrap_or(false),

        _ => false,
    }
}

fn value_cmp(lhs: &Value, rhs: &Value) -> Option<Ordering> {
    match (lhs, rhs) {
        (&Value::Scalar(ref x), &Value::Scalar(ref y)) => x.partial_cmp(y),
        (&Value::Array(ref x), &Value::Array(ref y)) => x.iter().partial_cmp(y.iter()),
        (&Value::Object(ref x), &Value::Object(ref y)) => x.iter().partial_cmp(y.iter()),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_string_scalar() {
        let val = Value::scalar(42f64);
        assert_eq!(&val.render().to_string(), "42");
        assert_eq!(&val.to_str(), "42");
    }

    #[test]
    fn test_to_string_array() {
        let val = Value::Array(vec![
            Value::scalar(3f64),
            Value::scalar("test"),
            Value::scalar(5.3),
        ]);
        assert_eq!(&val.render().to_string(), "3test5.3");
        assert_eq!(&val.to_str(), "3test5.3");
    }

    // TODO make a test for object, remember values are in arbitrary orders in HashMaps

    #[test]
    fn test_to_string_nil() {
        assert_eq!(&Value::nil().render().to_string(), "");
        assert_eq!(&Value::nil().to_str(), "");
    }

    #[test]
    fn scalar_equality() {
        assert_eq!(Value::scalar("alpha"), Value::scalar("alpha"));
        assert_eq!(Value::scalar(""), Value::scalar(""));
        assert!(Value::scalar("alpha") != Value::scalar("beta"));
        assert!(Value::scalar("beta") != Value::scalar("alpha"));
    }

    #[test]
    fn scalars_have_ruby_truthiness() {
        // all strings in ruby are true
        assert_eq!(Value::scalar(true), Value::scalar("All strings are truthy"));
        assert_eq!(Value::scalar(true), Value::scalar(""));
        assert!(Value::scalar("").is_truthy());

        assert_eq!(Value::scalar(true), Value::scalar(true));
        assert!(Value::scalar(true) != Value::scalar(false));
    }

    #[test]
    fn array_equality() {
        let a = Value::Array(vec![Value::scalar("one"), Value::scalar("two")]);
        let b = Value::Array(vec![Value::scalar("alpha"), Value::scalar("beta")]);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn arrays_have_ruby_truthiness() {
        assert_eq!(Value::scalar(true), Value::Array(Vec::new()));
        assert!(Value::Array(Vec::new()).is_truthy());
    }

    #[test]
    fn object_equality() {
        let a: Object = [
            ("alpha".into(), Value::scalar("1")),
            ("beta".into(), Value::scalar(2f64)),
        ]
        .into_iter()
        .cloned()
        .collect();
        let a = Value::Object(a);

        let b: Object = [
            ("alpha".into(), Value::scalar("1")),
            ("beta".into(), Value::scalar(2f64)),
            ("gamma".into(), Value::Array(vec![])),
        ]
        .into_iter()
        .cloned()
        .collect();
        let b = Value::Object(b);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn objects_have_ruby_truthiness() {
        assert_eq!(Value::scalar(true), Value::Object(Object::new()));
        assert!(Value::Object(Object::new()).is_truthy());
    }

    #[test]
    fn nil_equality() {
        assert_eq!(Value::Nil, Value::Nil);
    }

    #[test]
    fn nils_have_ruby_truthiness() {
        assert_eq!(Value::scalar(false), Value::Nil);
        assert!(!Value::Nil.is_truthy());

        assert_eq!(Value::scalar(false), Value::Nil);
        assert!(Value::scalar(true) != Value::Nil);
        assert!(Value::scalar("") != Value::Nil);
    }

    #[test]
    fn empty_equality() {
        // Truth table from https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        assert_eq!(Value::Empty, Value::Empty);
        assert_eq!(Value::Empty, Value::Blank);
        assert_eq!(Value::Empty, liquid_value!(""));
        assert_ne!(Value::Empty, liquid_value!(" "));
        assert_eq!(Value::Empty, liquid_value!([]));
        assert_ne!(Value::Empty, liquid_value!([nil]));
        assert_eq!(Value::Empty, liquid_value!({}));
        assert_ne!(Value::Empty, liquid_value!({ "a": nil }));
    }

    #[test]
    fn blank_equality() {
        // Truth table from https://stackoverflow.com/questions/885414/a-concise-explanation-of-nil-v-empty-v-blank-in-ruby-on-rails
        assert_eq!(Value::Blank, Value::Blank);
        assert_eq!(Value::Blank, Value::Empty);
        assert_eq!(Value::Blank, liquid_value!(nil));
        assert_eq!(Value::Blank, liquid_value!(false));
        assert_ne!(Value::Blank, liquid_value!(true));
        assert_ne!(Value::Blank, liquid_value!(0));
        assert_ne!(Value::Blank, liquid_value!(1));
        assert_eq!(Value::Blank, liquid_value!(""));
        assert_eq!(Value::Blank, liquid_value!(" "));
        assert_eq!(Value::Blank, liquid_value!([]));
        assert_ne!(Value::Blank, liquid_value!([nil]));
        assert_eq!(Value::Blank, liquid_value!({}));
        assert_ne!(Value::Blank, liquid_value!({ "a": nil }));
    }
}
