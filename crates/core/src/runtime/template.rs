use std::io::Write;

use crate::error::Result;
use crate::error::ResultLiquidReplaceExt;

use super::{Blankness, Renderable, Runtime};

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
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        for el in &self.elements {
            super::increment_render_ops(runtime, 1)?;

            match el.render_to(writer, runtime) {
                Ok(()) => {}
                Err(error) => {
                    if let Some(replacement) = super::handle_render_error(runtime, error)? {
                        writer
                            .write_all(replacement.as_bytes())
                            .replace("Failed to render")?;
                    }
                }
            }

            super::check_resource_limits(runtime)?;

            // Did the last element we processed set an interrupt? If so, we
            // need to abandon the rest of our child elements and just
            // return what we've got. This is usually in response to a
            // `break` or `continue` tag being rendered.
            if runtime
                .registers()
                .get_mut::<super::InterruptRegister>()
                .interrupted()
            {
                break;
            }
        }
        Ok(())
    }

    fn blankness(&self) -> Blankness {
        if self
            .elements
            .iter()
            .all(|element| element.blankness().is_blank())
        {
            Blankness::BlankNode
        } else {
            Blankness::NotBlank
        }
    }
}
