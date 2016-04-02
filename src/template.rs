use Renderable;
use context::Context;
use filters::{size, upcase, downcase, capitalize, minus, plus, times, divided_by, ceil, floor, round, prepend, append, first, last, pluralize, replace};
use error::Result;

pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {

        context.add_filter("size", Box::new(size));
        context.add_filter("upcase", Box::new(upcase));
        context.add_filter("downcase".to_owned(), Box::new(downcase));
        context.add_filter("capitalize".to_owned(), Box::new(capitalize));
        context.add_filter("minus", Box::new(minus));
        context.add_filter("plus", Box::new(plus));
        context.add_filter("times", Box::new(times));
        context.add_filter("divided_by", Box::new(divided_by));
        context.add_filter("ceil", Box::new(ceil));
        context.add_filter("floor", Box::new(floor));
        context.add_filter("round", Box::new(round));
        context.add_filter("first".to_owned(), Box::new(first));
        context.add_filter("last".to_owned(), Box::new(last));
        context.add_filter("prepend".to_owned(), Box::new(prepend));
        context.add_filter("append".to_owned(), Box::new(append));
        context.add_filter("replace", Box::new(replace));
        context.add_filter("pluralize".to_owned(), Box::new(pluralize));

        let mut buf = String::new();
        for el in &self.elements {
            if let Some(ref x) = try!(el.render(context)) {
                buf = buf + x;
            }

            // Did the last element we processed set an interrupt? If so, we
            // need to abandon the rest of our child elements and just
            // return what we've got. This is usually in response to a
            // `break` or `continue` tag being rendered.
            if context.interrupted() {
                break;
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
