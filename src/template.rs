use std::collections::HashMap;

use error::Result;
use super::Object;

use interpreter;
use interpreter::Renderable;

pub struct Template {
    pub(crate) template: interpreter::Template,
    pub(crate) filters: HashMap<String, interpreter::BoxedValueFilter>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &Object) -> Result<String> {
        let mut data = interpreter::Context::new()
            .with_filters(self.filters.clone())
            .with_values(globals.clone());
        let output = self.template
            .render(&mut data)?
            .expect("template never returns `None`");
        Ok(output)
    }
}
