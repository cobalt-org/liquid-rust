use std::collections::HashMap;
use std::io::Write;
use std::sync;

use super::Object;
use error::Result;

use interpreter;
use interpreter::Renderable;

pub struct Template {
    pub(crate) template: interpreter::Template,
    pub(crate) filters: sync::Arc<HashMap<&'static str, interpreter::BoxedValueFilter>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &Object) -> Result<String> {
        let mut data = Vec::new();
        self.render_to(&mut data, globals)?;
        Ok(String::from_utf8(data).expect("render only writes UTF-8"))
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut Write, globals: &Object) -> Result<()> {
        let mut data = interpreter::ContextBuilder::new()
            .set_filters(&self.filters)
            .set_globals(globals.clone())
            .build();
        self.template.render_to(writer, &mut data)
    }
}
