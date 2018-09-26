use std::io::Write;

use super::Context;
use super::Renderable;
use error::{Result, ResultLiquidChainExt};

/// A raw template expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    text: String,
}

impl Text {
    /// Create a raw template expression.
    pub fn new<S: Into<String>>(text: S) -> Text {
        Text { text: text.into() }
    }
}

impl Renderable for Text {
    fn render_to(&self, writer: &mut Write, _context: &mut Context) -> Result<()> {
        write!(writer, "{}", &self.text).chain("Failed to render")?;
        Ok(())
    }
}
