use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;
use std::borrow;

use super::Index;
use super::Scalar;

/// An enum to represent different value types
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Value {
    Scalar(Scalar),
    Array(Array),
    Object(Object),
    Nil,
}

/// Type representing a Liquid array, payload of the `Value::Array` variant
pub type Array = Vec<Value>;

/// Type representing a Liquid object, payload of the `Value::Object` variant
pub type Object = HashMap<String, Value>;

impl Value {
    pub fn scalar<T: Into<Scalar>>(value: T) -> Self {
        Value::Scalar(Scalar::new(value))
    }

    pub fn array<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let v: Array = iter.into_iter().collect();
        Value::Array(v)
    }

    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn to_str(&self) -> borrow::Cow<str> {
        match *self {
            Value::Scalar(ref x) => x.to_str(),
            Value::Array(ref x) => {
                let arr: Vec<String> = x.iter().map(|v| v.to_string()).collect();
                borrow::Cow::Owned(arr.join(", "))
            }
            Value::Object(ref x) => {
                let arr: Vec<String> = x.iter()
                    .map(|(k, v)| k.clone() + ": " + &v.to_string())
                    .collect();
                borrow::Cow::Owned(arr.join(", "))
            }
            Value::Nil => borrow::Cow::Borrowed(""),
        }
    }

    /// Extracts the scalar value if it is a scalar.
    pub fn as_scalar(&self) -> Option<&Scalar> {
        match *self {
            Value::Scalar(ref s) => Some(s),
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

    /// Tests whether this value is an object
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// Tests whether this value is nil
    pub fn is_nil(&self) -> bool {
        match *self {
            Value::Nil => true,
            _ => false,
        }
    }

    /// Evaluate using Liquid "truthiness"
    pub fn is_truthy(&self) -> bool {
        // encode Ruby truthiness: all values except false and nil are true
        match *self {
            Value::Scalar(ref x) => x.is_truthy(),
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn is_default(&self) -> bool {
        match *self {
            Value::Scalar(ref x) => x.is_default(),
            Value::Nil => true,
            Value::Array(ref x) => x.is_empty(),
            Value::Object(ref x) => x.is_empty(),
        }
    }

    pub fn get<'i, I: Into<&'i Index>>(&self, index: I) -> Option<&Self> {
        let index: &Index = index.into();
        self.get_index(index)
    }

    fn get_index(&self, index: &Index) -> Option<&Self> {
        match *self {
            Value::Array(ref x) => {
                if let Some(index) = index.as_index() {
                    let index = if 0 <= index {
                        index as isize
                    } else {
                        (x.len() as isize) + index
                    };
                    x.get(index as usize)
                } else {
                    None
                }
            }
            Value::Object(ref x) => {
                if let Some(key) = index.as_key() {
                    x.get(key)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Value::Scalar(ref x), &Value::Scalar(ref y)) => x == y,
            (&Value::Array(ref x), &Value::Array(ref y)) => x == y,
            (&Value::Object(ref x), &Value::Object(ref y)) => x == y,
            (&Value::Nil, &Value::Nil) => true,

            // encode Ruby truthiness: all values except false and nil are true
            (&Value::Nil, &Value::Scalar(ref b)) |
            (&Value::Scalar(ref b), &Value::Nil) => !b.to_bool().unwrap_or(true),
            (_, &Value::Scalar(ref b)) |
            (&Value::Scalar(ref b), _) => b.to_bool().unwrap_or(false),

            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&Value::Scalar(ref x), &Value::Scalar(ref y)) => x.partial_cmp(y),
            (&Value::Array(ref x), &Value::Array(ref y)) => x.iter().partial_cmp(y.iter()),
            (&Value::Object(ref x), &Value::Object(ref y)) => x.iter().partial_cmp(y.iter()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.to_str();
        write!(f, "{}", data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_string_scalar() {
        let val = Value::scalar(42f32);
        assert_eq!(&val.to_string(), "42");
    }

    #[test]
    fn test_to_string_array() {
        let val = Value::Array(vec![Value::scalar(3f32),
                                    Value::scalar("test"),
                                    Value::scalar(5.3)]);
        assert_eq!(&val.to_string(), "3, test, 5.3");
    }

    // TODO make a test for object, remember values are in arbitrary orders in HashMaps

    #[test]
    fn test_to_string_nil() {
        assert_eq!(&Value::nil().to_string(), "");
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
        let a: Object = [("alpha".to_owned(), Value::scalar("1")),
                         ("beta".to_owned(), Value::scalar(2f32))]
            .into_iter()
            .cloned()
            .collect();
        let a = Value::Object(a);

        let b: Object = [("alpha".to_owned(), Value::scalar("1")),
                         ("beta".to_owned(), Value::scalar(2f32)),
                         ("gamma".to_owned(), Value::Array(vec![]))]
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
}
