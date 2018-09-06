use std::io::Write;

use error::{Result, ResultLiquidChainExt};
use super::Context;
use super::Renderable;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    text: String,
}

impl Renderable for Text {
    fn render_to(&self, writer: &mut Write, _context: &mut Context) -> Result<()> {
        write!(writer, "{}", &self.text).chain("Failed to render")?;
        Ok(())
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_owned(),
        }
    }
}
