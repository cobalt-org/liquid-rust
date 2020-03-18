use std::io::Write;

use liquid_error::{Result, ResultLiquidReplaceExt};
use crate::runtime::Renderable;
use crate::runtime::Runtime;

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
    fn render_to(&self, writer: &mut dyn Write, _runtime: &mut Runtime) -> Result<()> {
        write!(writer, "{}", &self.text).replace("Failed to render")?;
        Ok(())
    }
}
