use Renderable;
use context::Context;
use filters::{size, upcase, minus, plus, replace};
use error::Result;

pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        context.add_filter("size",    Box::new(size));
        context.add_filter("upcase",  Box::new(upcase));
        context.add_filter("minus",   Box::new(minus));
        context.add_filter("plus",    Box::new(plus));
        context.add_filter("replace", Box::new(replace));

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
