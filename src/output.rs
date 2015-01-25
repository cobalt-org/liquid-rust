use Renderable;
use Context;
use Value;

pub struct FilterPrototype{
    name: String,
    arguments: Vec<Value>
}

impl FilterPrototype {
    pub fn new(name: &str, arguments: Vec<Value>) -> FilterPrototype {
        FilterPrototype{name: name.to_string(), arguments: arguments}
    }
}

pub struct Output{
    name: String,
    filters: Vec<FilterPrototype>
}

impl Renderable for Output {
    fn render (&self, context: &mut Context) -> Option<String>{
        match context.values.get(&self.name) {
            Some(val) => Some(val.to_string()),
            None => None
        }
    }
}

impl Output {
    pub fn new(name: &str, filters: Vec<FilterPrototype>) -> Output {
        Output{name: name.to_string(), filters: filters}
    }
}

