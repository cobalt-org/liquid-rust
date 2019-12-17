use std::io::Write;

use liquid_error::Result;

use super::Context;
use super::Renderable;

/// An executable template block.
#[derive(Debug)]
pub struct Template {
    elements: Vec<Box<dyn Renderable>>,
}

impl Template {
    /// Create an executable template block.
    pub fn new(elements: Vec<Box<dyn Renderable>>) -> Template {
        Template { elements }
    }
}

impl Renderable for Template {
    fn render_to(&self, writer: &mut dyn Write, context: &mut Context<'_>) -> Result<()> {
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
