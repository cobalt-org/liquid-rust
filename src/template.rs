use Renderable;
use context::Context;
use filters::{size, upcase};
use error::Result;

pub struct Template<'a> {
    pub elements: Vec<Box<Renderable + 'a>>,
}

impl<'a> Renderable for Template<'a> {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        context.filters.insert("size".to_string(), Box::new(size));
        context.filters.insert("upcase".to_string(), Box::new(upcase));

        let mut buf = String::new();
        for el in self.elements.iter() {
            if let Some(ref x) = try!(el.render(context)) {
                buf = buf + x;
            }
        }
        Ok(Some(buf))
    }
}

impl<'a> Template<'a> {
    pub fn new(elements: Vec<Box<Renderable + 'a>>) -> Template<'a> {
        Template { elements: elements }
    }
}
