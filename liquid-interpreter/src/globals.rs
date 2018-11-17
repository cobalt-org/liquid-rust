use std::fmt;

use error::{Error, Result};
use value::Object;
use value::Value;

/// Immutable view into a template's global variables.
pub trait Globals: fmt::Debug {
    /// Access a global variable.
    fn get<'a>(&'a self, name: &str) -> Result<&'a Value>;
}

impl Globals for Object {
    fn get<'a>(&'a self, name: &str) -> Result<&'a Value> {
        self.get(name)
            .ok_or_else(|| Error::with_msg("Unknown variable").context("variable", name.to_owned()))
    }
}
