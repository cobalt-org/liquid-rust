use Renderable;
use Context;

pub struct Output{
    name: String
}

impl Renderable for Output {
    fn render (&self, context: &Context) -> Option<String>{
        match context.values.get(&self.name) {
            Some(val) => Some(val.to_string()),
            None => None
        }
    }
}

impl Output {
    pub fn new(name: &str) -> Output {
        Output{name: name.to_string()}
    }
}

