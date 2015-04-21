use Renderable;
use context::Context;

#[derive(Debug)]
pub struct Variable{
    name: String
}

impl Renderable for Variable {
    fn render (&self, context: &mut Context) -> Option<String>{
        match context.get_val(&self.name) {
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

