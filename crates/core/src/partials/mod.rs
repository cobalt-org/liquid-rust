use std::borrow;
use std::fmt;
use std::sync;

use crate::error::Result;
use crate::parser::Language;
use crate::runtime::PartialStore;

mod eager;
mod inmemory;
mod lazy;
mod ondemand;

pub use self::eager::*;
pub use self::inmemory::*;
pub use self::lazy::*;
pub use self::ondemand::*;

/// Compile a `PartialSource` into a `PartialStore` of `Renderable`s.
///
/// This trait is intended to allow a variety of implementation/policies to fit your needs,
/// including:
/// - Compile partials eagerly or lazily.
/// - Report compile errors eagerly or lazily.
/// - Whether to cache the results or not.
pub trait PartialCompiler {
    /// Convert a `PartialSource` into a `PartialStore`.
    fn compile(self, language: sync::Arc<Language>) -> Result<Box<dyn PartialStore + Send + Sync>>;

    /// Access underlying `PartialSource`
    fn source(&self) -> &dyn PartialSource;
}

/// Partial-template source repository.
pub trait PartialSource: fmt::Debug {
    /// Enumerate all partial-templates.
    fn names(&self) -> Vec<&str>;

    /// Access a partial-template.
    fn get<'a>(&'a self, _name: &str) -> Result<Option<borrow::Cow<'a, str>>> {
        Ok(None)
    }
}
