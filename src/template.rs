use std::collections::HashMap;

use error::Result;
use super::Object;

use syntax;
use syntax::Renderable;

pub struct Template {
    pub(crate) template: syntax::Template,
    pub(crate) filters: HashMap<String, syntax::BoxedValueFilter>,
}

impl Template {
    pub fn render(&self, globals: &Object) -> Result<String> {
        let mut data = syntax::Context::new()
            .with_filters(self.filters.clone())
            .with_values(globals.clone());
        let output = self.template
            .render(&mut data)?
            .expect("template never returns `None`");
        Ok(output)
    }
}
