use Renderable;
use context::Context;
use std::collections::HashMap;
use std::cmp::Ordering;
use error::Result;

/// An enum to represent different value types
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Num(f32),
    Str(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Bool(bool)
}

impl Value {
    pub fn str(val: &str) -> Value {
        Value::Str(val.to_owned())
    }

    pub fn is_truthy(&self) -> bool {
        match *self {
            Value::Bool(x) => x,
            _ => true
        }
    }

    pub fn is_falsey(&self) -> bool {
        !self.is_truthy()
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
            },
            Value::Object(ref x) => {
                let arr: Vec<String> = x.iter().map(|(k, v)| k.clone() + ": " + &v.to_string()).collect();
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
        let val = Value::Array(vec![Value::Num(3f32), Value::Str("test".to_owned()), Value::Num(5.3)]);
        assert_eq!(&val.to_string(), "3, test, 5.3");
    }

    #[test]
    fn booleans_have_ruby_truthiness() {
        assert_eq!(true, Value::Bool(true).is_truthy());
        assert_eq!(true, Value::Bool(false).is_falsey());

        assert_eq!(false, Value::Bool(true).is_falsey());
        assert_eq!(false, Value::Bool(false).is_truthy());
    }

    #[test]
    fn strings_have_ruby_truthiness() {
        assert_eq!(true, Value::str("All strings are truthy").is_truthy());
        assert_eq!(true, Value::str("").is_truthy());
    }

    #[test]
    fn numbers_have_ruby_truthiness() {
        assert_eq!(true, Value::Num(42f32).is_truthy());
        assert_eq!(true, Value::Num(0f32).is_truthy());
    }

    // TODO make a test for object, remember values are in arbitrary orders in HashMaps
}
