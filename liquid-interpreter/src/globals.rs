use std::fmt;

use value::Object;
use value::Value;

/// Immutable view into a template's global variables.
pub trait Globals: fmt::Debug {
    /// Access a global variable.
    fn get<'a>(&'a self, name: &str) -> Option<&'a Value>;
}

impl Globals for Object {
    fn get<'a>(&'a self, name: &str) -> Option<&'a Value> {
        self.get(name)
    }
}
