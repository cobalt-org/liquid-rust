use Renderable;
use Value;
use std::collections::HashMap;

pub struct Template<'a>{
    pub elements: Vec<Box<Renderable +'a>>
}

impl<'a> Renderable for Template<'a> {
    fn render (&self, context: &HashMap<String, Value>) -> Option<String>{
        Some(self.elements.iter().fold(String::new(), |fold, val| {
                                  match val.render(context)  {
                                      Some(x) => fold + x.as_slice(),
                                      _ => fold
                                  }
                                 }))
    }
}

impl<'a> Template<'a> {
    pub fn new(elements: Vec<Box<Renderable +'a>>) -> Template<'a> {
        Template{elements: elements}
    }
}

