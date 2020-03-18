use std::fmt::Debug;
use std::io::Write;

use liquid_error::Result;

use super::Runtime;

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable: Send + Sync + Debug {
    /// Renders the Renderable instance given a Liquid runtime.
    fn render(&self, runtime: &mut Runtime<'_>) -> Result<String> {
        let mut data = Vec::new();
        self.render_to(&mut data, runtime)?;
        Ok(String::from_utf8(data).expect("render only writes UTF-8"))
    }

    /// Renders the Renderable instance given a Liquid runtime.
    fn render_to(&self, writer: &mut dyn Write, runtime: &mut Runtime<'_>) -> Result<()>;
}
