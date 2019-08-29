use std::fmt::Debug;
use std::io::Write;

use liquid_error::Result;

use super::Context;

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable: Send + Sync + Debug {
    /// Renders the Renderable instance given a Liquid context.
    fn render(&self, context: &mut Context) -> Result<String> {
        let mut data = Vec::new();
        self.render_to(&mut data, context)?;
        Ok(String::from_utf8(data).expect("render only writes UTF-8"))
    }

    /// Renders the Renderable instance given a Liquid context.
    fn render_to(&self, writer: &mut dyn Write, context: &mut Context) -> Result<()>;
}
