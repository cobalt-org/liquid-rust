use Renderable;
use context::Context;
use std::collections::HashMap;
use std::cmp::Ordering;

/// An enum to represent different value types
#[derive(Clone, PartialEq, Debug)]
pub enum Value{
    Num(f32),
    Str(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>)
}

// TODO implement for object and array
// TODO clean this up
impl PartialOrd<Value> for Value{
    fn partial_cmp(&self, other: &Value) -> Option<Ordering>{
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.partial_cmp(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.partial_cmp(y),
            _ => None
        }
    }
    fn lt(&self, other: &Value) -> bool{
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.lt(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.lt(y),
            _ => false
        }
    }
    fn le(&self, other: &Value) -> bool{
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.le(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.le(y),
            _ => false
        }
    }
    fn gt(&self, other: &Value) -> bool{
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.gt(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.gt(y),
            _ => false
        }
    }
    fn ge(&self, other: &Value) -> bool{
        match (self, other) {
            (&Value::Num(x), &Value::Num(y)) => x.ge(&y),
            (&Value::Str(ref x), &Value::Str(ref y)) => x.ge(y),
            _ => false
        }
    }
}

impl ToString for Value{
    fn to_string(&self) -> String{
        match self{
            &Value::Num(ref x) => x.to_string(),
            &Value::Str(ref x) => x.to_string(),
            _ => "[Object object]".to_string() // TODO
        }
    }
}

impl Renderable for Value{
    fn render(&self, _context: &mut Context) -> Option<String>{
        Some(self.to_string())
    }
}
