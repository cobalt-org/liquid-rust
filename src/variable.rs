use Renderable;
use std::collections::HashMap;

pub struct Variable{
    name: String
}

impl Renderable for Variable {
    fn render (&self, context: &HashMap<String, String>) -> String{
        match context.find(&self.name) {
            Some(val) => val.to_string(),
            None => "".to_string()
        }
    }
}

impl Variable {
    pub fn new(name: &str) -> Variable {
        Variable{name: name.to_string()}
    }
}

