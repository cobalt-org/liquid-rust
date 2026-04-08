use std::fmt::Debug;
use std::io::Write;

use crate::error::Result;

use super::Runtime;

/// Parse-time blankness metadata for a renderable node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blankness {
    /// The node may produce visible output and must keep surrounding whitespace.
    NotBlank,
    /// The node is a whitespace-only text node that can be stripped from blank block bodies.
    BlankText,
    /// The node renders no visible output but may still have side effects.
    BlankNode,
}

impl Blankness {
    /// Returns `true` when the node is considered blank during parse-time compatibility checks.
    pub fn is_blank(self) -> bool {
        !matches!(self, Self::NotBlank)
    }

    /// Returns `true` when the node is a whitespace-only text node.
    pub fn is_blank_text(self) -> bool {
        matches!(self, Self::BlankText)
    }
}

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable: Send + Sync + Debug {
    /// Renders the Renderable instance given a Liquid runtime.
    fn render(&self, runtime: &dyn Runtime) -> Result<String> {
        let mut data = Vec::new();
        self.render_to(&mut data, runtime)?;
        Ok(String::from_utf8(data).expect("render only writes UTF-8"))
    }

    /// Renders the Renderable instance given a Liquid runtime.
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()>;

    /// Returns parse-time blankness for compatibility behaviors like stripping
    /// whitespace-only text from blank control-flow bodies.
    fn blankness(&self) -> Blankness {
        Blankness::NotBlank
    }
}
