use Renderable;
use context::Context;
use std::collections::HashMap;
use std::cmp::Ordering;
use error::{Result, Error};
use token::Token;
use token::Token::*;

/// An enum to represent different value types
#[derive(Clone, Debug)]
pub enum Value {
    Num(f32),
    Str(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool),
}

impl<'a> Value {
    /// Shorthand function to create Value::Str from a string slice.
    pub fn str(val: &str) -> Value {
        Value::Str(val.to_owned())
    }

    pub fn as_str(&'a self) -> Option<&'a str> {
        match *self {
            Value::Str(ref v) => Some(v),
            _ => None,
        }
    }

    /// Parses a token that can possibly represent a Value
    /// to said Value. Returns an Err if the token can not
    /// be interpreted as a Value.
    pub fn from_token(t: &Token) -> Result<Value> {
        match t {
            &StringLiteral(ref x) => Ok(Value::Str(x.to_owned())),
            &NumberLiteral(x) => Ok(Value::Num(x)),
            &BooleanLiteral(x) => Ok(Value::Bool(x)),
            x => Error::parser("Value", Some(x)),
        }
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x == y,
            (&Value::Str(ref x), &Value::Str(ref y)) => x == y,
            (&Value::Bool(x), &Value::Bool(y)) => x == y,
            (&Value::Object(ref x), &Value::Object(ref y)) => x == y,
            (&Value::Array(ref x), &Value::Array(ref y)) => x == y,

            // encode Ruby truthiness; all values except false and nil
            // are true, and we don't have a notion of nil
            (_, &Value::Bool(b)) |
            (&Value::Bool(b), _) => b,

            _ => false,
        }
    }
}

// TODO implement for object and array
// TODO clean this up
impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.partial_cmp(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.partial_cmp(y),
            (&Value::Bool(x), &Value::Bool(y)) => x.partial_cmp(&y),
            _ => None,
        }
    }
    fn lt(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.lt(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.lt(y),
            (&Value::Bool(x), &Value::Bool(y)) => x.lt(&y),
            _ => false,
        }
    }
    fn le(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.le(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.le(y),
            (&Value::Bool(x), &Value::Bool(y)) => x.le(&y),
            _ => false,
        }
    }
    fn gt(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.gt(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.gt(y),
            (&Value::Bool(x), &Value::Bool(y)) => x.gt(&y),
            _ => false,
        }
    }
    fn ge(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.ge(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.ge(y),
            (&Value::Bool(x), &Value::Bool(y)) => x.ge(&y),
            _ => false,
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match *self {
            Value::Bool(ref x) => x.to_string(),
            Value::Num(ref x) => x.to_string(),
            Value::Str(ref x) => x.to_owned(),
            Value::Array(ref x) => {
                let arr: Vec<String> = x.iter().map(|v| v.to_string()).collect();
                arr.join(", ")
            }
            Value::Object(ref x) => {
                let arr: Vec<String> =
                    x.iter().map(|(k, v)| k.clone() + ": " + &v.to_string()).collect();
                arr.join(", ")
            }
        }
    }
}

impl Renderable for Value {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        Ok(Some(self.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    static TRUE: Value = Value::Bool(true);
    static FALSE: Value = Value::Bool(false);

    #[test]
    fn test_as_str() {
        let val = Value::Num(42f32);
        assert_eq!(val.as_str(), None);

        let val = Value::str("test");
        assert_eq!(val.as_str(), Some("test"));
    }

    #[test]
    fn test_num_to_string() {
        let val = Value::Num(42f32);
        assert_eq!(&val.to_string(), "42");

        let val = Value::Num(42.34);
        assert_eq!(&val.to_string(), "42.34");
    }

    #[test]
    fn test_str_to_string() {
        let val = Value::Str("foobar".to_owned());
        assert_eq!(&val.to_string(), "foobar");
    }

    #[test]
    fn test_array_to_string() {
        let val =
            Value::Array(vec![Value::Num(3f32), Value::Str("test".to_owned()), Value::Num(5.3)]);
        assert_eq!(&val.to_string(), "3, test, 5.3");
    }

    // TODO make a test for object, remember values are in arbitrary orders in HashMaps

    #[test]
    fn boolean_equality() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Bool(false), Value::Bool(false));
        assert!(Value::Bool(false) != Value::Bool(true));
        assert!(Value::Bool(true) != Value::Bool(false));
    }

    #[test]
    fn booleans_have_ruby_truthiness() {
        assert_eq!(TRUE, Value::Bool(true));
        assert_eq!(FALSE, Value::Bool(false));
    }

    #[test]
    fn string_equality() {
        assert_eq!(Value::str("alpha"), Value::str("alpha"));
        assert_eq!(Value::str(""), Value::str(""));
        assert!(Value::str("alpha") != Value::str("beta"));
        assert!(Value::str("beta") != Value::str("alpha"));
    }

    #[test]
    fn strings_have_ruby_truthiness() {
        // all strings in ruby are true
        assert_eq!(TRUE, Value::str("All strings are truthy"));
        assert_eq!(TRUE, Value::str(""));
    }

    #[test]
    fn number_equality() {
        assert_eq!(Value::Num(42f32), Value::Num(42f32));
        assert_eq!(Value::Num(0f32), Value::Num(0f32));
        assert!(Value::Num(1f32) != Value::Num(2f32));
        assert!(Value::Num(2f32) != Value::Num(1f32));
    }

    #[test]
    fn numbers_have_ruby_truthiness() {
        assert_eq!(TRUE, Value::Num(42f32));
        assert_eq!(TRUE, Value::Num(0f32));
    }

    #[test]
    fn object_equality() {
        let mut values = HashMap::<String, Value>::new();
        values.insert("alpha".to_owned(), Value::str("1"));
        values.insert("beta".to_owned(), Value::Num(2f32));

        let a = Value::Object(values.clone());

        values.insert("gamma".to_owned(), Value::Array(vec![]));
        let b = Value::Object(values);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn objects_have_ruby_truthiness() {
        assert_eq!(TRUE, Value::Object(HashMap::new()));
    }


    #[test]
    fn array_equality() {
        let a = Value::Array(vec![Value::str("one"), Value::str("two")]);
        let b = Value::Array(vec![Value::str("alpha"), Value::str("beta")]);

        assert_eq!(a, a);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn arrays_have_ruby_truthiness() {
        assert_eq!(TRUE, Value::Array(Vec::new()));
    }

    #[test]
    fn mixed_comparisons_are_false() {
        // assers that all comparisons between different types of values
        // are false
        let mut values = HashMap::<String, Value>::new();
        values.insert("alpha".to_owned(), Value::str("1"));

        let terms = vec![Value::Num(1f32),
                         Value::str("1"),
                         Value::Object(values),
                         Value::Array(vec![Value::Num(1f32)])];

        for (x, a) in terms.iter().enumerate() {
            for (y, b) in terms.iter().enumerate() {
                if x != y {
                    assert!(a != b);
                    assert!(b != a);
                }
            }
        }
    }
}
