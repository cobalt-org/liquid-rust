use error::Result;

use super::Context;

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable: Send + Sync {
    /// Renders the Renderable instance given a Liquid context.
    /// The Result that is returned signals if there was an error rendering,
    /// the Option<String> that is wrapped by the Result will be None if
    /// the render has run successfully but there is no content to render.
    fn render(&self, context: &mut Context) -> Result<Option<String>>;
}
