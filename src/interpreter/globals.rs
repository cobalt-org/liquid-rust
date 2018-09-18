use std::fmt;

use value::Object;
use value::Value;

pub trait Globals: fmt::Debug {
    fn get<'a>(&'a self, name: &str) -> Option<&'a Value>;
}

impl Globals for Object {
    fn get<'a>(&'a self, name: &str) -> Option<&'a Value> {
        self.get(name)
    }
}
