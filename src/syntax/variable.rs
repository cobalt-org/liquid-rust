use error::Result;

use super::Context;
use super::Renderable;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    name: String,
}

impl Renderable for Variable {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let res = match context.get_val(&self.name) {
            Some(val) => Some(val.to_string()),
            None => None,
        };

        Ok(res)
    }
}

impl Variable {
    pub fn new(name: &str) -> Variable {
        Variable { name: name.to_owned() }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
