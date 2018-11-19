use std::fmt;

use error::{Error, Result};
use value::Object;
use value::Path;
use value::Value;

/// Immutable view into a template's global variables.
pub trait Globals: fmt::Debug {
    /// Check if global variable exists.
    fn contains_global(&self, name: &str) -> bool;

    /// Enumerate all globals
    fn globals(&self) -> Vec<&str>;

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
    fn try_get_variable<'a>(&'a self, path: &Path) -> Option<&'a Value>;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn get_variable<'a>(&'a self, path: &Path) -> Result<&'a Value>;
}

impl Globals for Object {
    fn contains_global(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    fn globals(&self) -> Vec<&str> {
        self.keys().map(|s| s.as_ref()).collect()
    }

    fn contains_variable(&self, path: &Path) -> bool {
        get_variable_option(self, path).is_some()
    }

    fn try_get_variable<'a>(&'a self, path: &Path) -> Option<&'a Value> {
        get_variable_option(self, path)
    }

    fn get_variable<'a>(&'a self, path: &Path) -> Result<&'a Value> {
        self.try_get_variable(path).ok_or_else(|| {
            Error::with_msg("Unknown index").context("variable", format!("{}", path))
        })
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
