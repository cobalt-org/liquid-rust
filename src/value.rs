use Renderable;
use context::Context;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Value{
    Num(f32),
    Str(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>)
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
