use Renderable;
use Value;
use std::collections::HashMap;

pub struct Variable{
    name: String
}

impl Renderable for Variable {
    fn render (&self, context: &HashMap<String, Value>) -> Option<String>{
        match context.get(&self.name) {
            Some(val) => Some(val.to_string()),
            None => None
        }
    }
}

impl Variable {
    pub fn new(name: &str) -> Variable {
        Variable{name: name.to_string()}
    }
}

