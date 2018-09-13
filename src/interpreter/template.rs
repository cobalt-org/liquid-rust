use std::io::Write;

use super::Context;
use super::Renderable;
use error::Result;

#[derive(Debug)]
pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        for el in &self.elements {
            el.render_to(writer, context)?;

            // Did the last element we processed set an interrupt? If so, we
            // need to abandon the rest of our child elements and just
            // return what we've got. This is usually in response to a
            // `break` or `continue` tag being rendered.
            if context.interrupt().interrupted() {
                break;
            }
        }
        Ok(())
    }
}

impl Template {
    pub fn new(elements: Vec<Box<Renderable>>) -> Template {
        Template { elements }
    }
}
