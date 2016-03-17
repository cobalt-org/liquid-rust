use Renderable;
use context::Context;
use filters::{size, upcase, minus, plus, replace};
use error::Result;

pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        context.filters.insert("size".to_owned(), Box::new(size));
        context.filters.insert("upcase".to_owned(), Box::new(upcase));
        context.filters.insert("minus".to_owned(), Box::new(minus));
        context.filters.insert("plus".to_owned(), Box::new(plus));
        context.filters.insert("replace".to_owned(), Box::new(replace));

        let mut buf = String::new();
        for el in &self.elements {
            if let Some(ref x) = try!(el.render(context)) {
                buf = buf + x;
            }
        }
        Ok(Some(buf))
    }
}

impl Template {
    pub fn new(elements: Vec<Box<Renderable>>) -> Template {
        Template { elements: elements }
    }
}
