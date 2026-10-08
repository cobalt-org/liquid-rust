use std::io::Write;
use std::sync;

use liquid_core::error::Result;
use liquid_core::runtime;
use liquid_core::runtime::PartialStore;
use liquid_core::runtime::Renderable;
use liquid_core::runtime::RenderingMode;

pub struct Template {
    pub(crate) template: runtime::Template,
    pub(crate) partials: Option<sync::Arc<dyn PartialStore + Send + Sync>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &dyn crate::ObjectView) -> Result<String> {
        self.render_with_mode(globals, RenderingMode::Strict)
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut dyn Write, globals: &dyn crate::ObjectView) -> Result<()> {
        self.render_to_with_mode(writer, globals, RenderingMode::Strict)
    }

    /// Renders an instance of the Template, using the given globals in lax mode.
    pub fn render_lax(&self, globals: &dyn crate::ObjectView) -> Result<String> {
        self.render_with_mode(globals, RenderingMode::Lax)
    }

    /// Renders an instance of the Template, using the given globals in lax mode.
    pub fn render_to_lax(
        &self,
        writer: &mut dyn Write,
        globals: &dyn crate::ObjectView,
    ) -> Result<()> {
        self.render_to_with_mode(writer, globals, RenderingMode::Lax)
    }

    /// Renders an instance of the Template, using the given globals with the provided rendering mode.
    fn render_with_mode(
        &self,
        globals: &dyn crate::ObjectView,
        mode: RenderingMode,
    ) -> Result<String> {
        const BEST_GUESS: usize = 10_000;
        let mut data = Vec::with_capacity(BEST_GUESS);
        self.render_to_with_mode(&mut data, globals, mode)?;

        Ok(convert_buffer(data))
    }

    /// Renders an instance of the Template, using the given globals with the provided rendering mode.
    fn render_to_with_mode(
        &self,
        writer: &mut dyn Write,
        globals: &dyn crate::ObjectView,
        mode: RenderingMode,
    ) -> Result<()> {
        let runtime = runtime::RuntimeBuilder::new()
            .set_globals(globals)
            .set_render_mode(mode);
        let runtime = match self.partials {
            Some(ref partials) => runtime.set_partials(partials.as_ref()),
            None => runtime,
        };
        let runtime = runtime.build();
        self.template.render_to(writer, &runtime)
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
