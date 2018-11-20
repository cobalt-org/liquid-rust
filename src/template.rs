use std::collections::HashMap;
use std::io::Write;
use std::sync;

use liquid_error::Result;
use liquid_interpreter as interpreter;
use liquid_interpreter::Renderable;

pub struct Template {
    pub(crate) template: interpreter::Template,
    pub(crate) filters: sync::Arc<HashMap<&'static str, interpreter::BoxedValueFilter>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &interpreter::Globals) -> Result<String> {
        const BEST_GUESS: usize = 10_000;
        let mut data = Vec::with_capacity(BEST_GUESS);
        self.render_to(&mut data, globals)?;

        Ok(convert_buffer(data))
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut Write, globals: &interpreter::Globals) -> Result<()> {
        let mut data = interpreter::ContextBuilder::new()
            .set_filters(&self.filters)
            .set_globals(globals)
            .build();
        self.template.render_to(writer, &mut data)
    }
}

#[cfg(debug_assertions)]
fn convert_buffer(buffer: Vec<u8>) -> String {
    String::from_utf8(buffer)
        .expect("render can only write UTF-8 because all inputs and processing preserve utf-8")
}

#[cfg(not(debug_assertions))]
fn convert_buffer(buffer: Vec<u8>) -> String {
    unsafe { String::from_utf8_unchecked(buffer) }
}
