use std::io::Write;
use std::sync;

use liquid_error::Result;
use liquid_interpreter as interpreter;
use liquid_interpreter::PartialStore;
use liquid_interpreter::Renderable;

pub struct Template {
    pub(crate) template: interpreter::Template,
    pub(crate) partials: Option<sync::Arc<PartialStore + Send + Sync>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &interpreter::ValueStore) -> Result<String> {
        const BEST_GUESS: usize = 10_000;
        let mut data = Vec::with_capacity(BEST_GUESS);
        self.render_to(&mut data, globals)?;

        Ok(convert_buffer(data))
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut Write, globals: &interpreter::ValueStore) -> Result<()> {
        let context = interpreter::ContextBuilder::new().set_globals(globals);
        let context = match self.partials {
            Some(ref partials) => context.set_partials(partials.as_ref()),
            None => context,
        };
        let mut context = context.build();
        self.template.render_to(writer, &mut context)
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
