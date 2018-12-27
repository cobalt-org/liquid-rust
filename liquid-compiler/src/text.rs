use std::io::Write;

use liquid_error::{Result, ResultLiquidReplaceExt};
use liquid_interpreter::Context;
use liquid_interpreter::Renderable;

/// A raw template expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Text {
    text: String,
}

impl Text {
    /// Create a raw template expression.
    pub(crate) fn new<S: Into<String>>(text: S) -> Text {
        Text { text: text.into() }
    }
}

impl Renderable for Text {
    fn render_to(&self, writer: &mut Write, _context: &mut Context) -> Result<()> {
        write!(writer, "{}", &self.text).replace("Failed to render")?;
        Ok(())
    }
}
