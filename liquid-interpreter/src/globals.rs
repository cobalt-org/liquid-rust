use std::fmt;

use value::Object;
use value::Path;
use value::Value;

/// Immutable view into a template's global variables.
pub trait Globals: fmt::Debug {
    /// Check if global variable exists.
    fn contains_global(&self, name: &str) -> bool;

    /// Access a global variable.
    fn get_global<'a>(&'a self, name: &str) -> Option<&'a Value>;

    /// Check if variable exists.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn contains_variable(&self, path: &Path) -> bool;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn get_variable<'a>(&'a self, path: &Path) -> Option<&'a Value>;
}

impl Globals for Object {
    fn contains_global(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    fn get_global<'a>(&'a self, name: &str) -> Option<&'a Value> {
        self.get(name)
    }

    fn contains_variable(&self, path: &Path) -> bool {
        get_variable_option(self, path).is_some()
    }

    fn get_variable<'a>(&'a self, path: &Path) -> Option<&'a Value> {
        get_variable_option(self, path)
    }
}

fn get_variable_option<'o>(obj: &'o Object, path: &Path) -> Option<&'o Value> {
    let mut indexes = path.iter();
    let key = indexes.next()?;
    let key = key.to_str();
    let value = obj.get(key.as_ref())?;

    indexes.fold(Some(value), |value, index| {
        let value = value?;
        value.get(index)
    })
}
