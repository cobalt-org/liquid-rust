use std::fmt;
use std::sync;

use crate::error::Result;

use super::Renderable;

/// Available partial-templates for including.
pub trait PartialStore: fmt::Debug {
    /// Enumerate all partial-templates.
    fn names(&self) -> Vec<&str>;

    /// Access a partial-template.
    fn get(&self, name: &str) -> Result<Option<sync::Arc<dyn Renderable>>>;
}
