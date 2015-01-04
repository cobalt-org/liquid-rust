use Renderable;
use Context;

pub struct Variable{
    name: String
}

impl Renderable for Variable {
    fn render (&self, context: &Context) -> Option<String>{
        match context.values.get(&self.name) {
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

