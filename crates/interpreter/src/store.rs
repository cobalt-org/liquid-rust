use std::fmt;

use itertools;
use liquid_error::{Error, Result};
use liquid_value::Object;
use liquid_value::PathRef;
use liquid_value::ScalarCow;
use liquid_value::Value;

/// Immutable view into a template's global variables.
pub trait ValueStore: fmt::Debug {
    /// Check if root variable exists.
    fn contains_root(&self, name: &str) -> bool;

    /// Enumerate all root variables.
    fn roots(&self) -> Vec<sstring::SStringRef<'_>>;

    /// Check if variable exists.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn contains_variable(&self, path: PathRef<'_, '_>) -> bool;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn try_get_variable<'a>(&'a self, path: PathRef<'_, '_>) -> Option<&'a Value>;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn get_variable<'a>(&'a self, path: PathRef<'_, '_>) -> Result<&'a Value>;
}

impl ValueStore for Object {
    fn contains_root(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    fn roots(&self) -> Vec<sstring::SStringRef<'_>> {
        self.keys().map(|s| s.as_ref()).collect()
    }

    fn contains_variable(&self, path: PathRef<'_, '_>) -> bool {
        get_variable_option(self, path).is_some()
    }

    fn try_get_variable<'a>(&'a self, path: PathRef<'_, '_>) -> Option<&'a Value> {
        get_variable_option(self, path)
    }

    fn get_variable<'a>(&'a self, path: PathRef<'_, '_>) -> Result<&'a Value> {
        if let Some(res) = self.try_get_variable(path) {
            return Ok(res);
        } else {
            for cur_idx in 1..path.len() {
                let subpath_end = path.len() - cur_idx;
                let subpath = &path[0..subpath_end];
                if let Some(parent) = self.try_get_variable(subpath) {
                    let subpath = itertools::join(subpath.iter().map(ScalarCow::render), ".");
                    let requested = &path[subpath_end];
                    let available = if let Some(arr) = parent.as_array() {
                        let mut available = vec![ScalarCow::new("first"), ScalarCow::new("last")];
                        if !arr.is_empty() {
                            available.insert(0, ScalarCow::new(format!("0..{}", arr.len() - 1)));
                        }
                        available
                    } else {
                        let available: Vec<_> = parent.keys().collect();
                        available
                    };
                    let available = itertools::join(available.iter().map(ScalarCow::render), ", ");
                    return Error::with_msg("Unknown index")
                        .context("variable", subpath)
                        .context("requested index", format!("{}", requested.render()))
                        .context("available indexes", available)
                        .into_err();
                }
            }

            let requested = path
                .get(0)
                .expect("`Path` guarantees at least one element")
                .to_sstr()
                .into_owned();
            let available = itertools::join(self.keys(), ", ");
            return Error::with_msg("Unknown variable")
                .context("requested variable", requested)
                .context("available variables", available)
                .into_err();
        }
    }
}

fn get_variable_option<'o>(obj: &'o Object, path: PathRef<'_, '_>) -> Option<&'o Value> {
    let mut indexes = path.iter();
    let key = indexes.next()?;
    let key = key.to_sstr();
    let value = obj.get(key.as_str())?;

    indexes.fold(Some(value), |value, index| {
        let value = value?;
        value.get(index)
    })
}
