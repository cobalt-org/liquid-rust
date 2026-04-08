use std::io;
use std::io::Write;
use std::sync;

use liquid_core::error::Result;
use liquid_core::runtime;
use liquid_core::runtime::PartialStore;
use liquid_core::runtime::Renderable;
use liquid_core::Error;
use liquid_core::Runtime;

/// Controls error behavior for configurable rendering.
#[derive(Clone, Copy)]
pub enum ErrorMode {
    /// Abort on the first render error.
    Strict,
    /// Format errors inline and continue rendering.
    Lenient(fn(&Error) -> String),
}

impl Default for ErrorMode {
    fn default() -> Self {
        Self::Lenient(default_error_formatter)
    }
}

/// Public render configuration for a single render call.
#[derive(Clone, Copy, Default)]
pub struct RenderOptions {
    pub max_output_bytes: Option<usize>,
    pub max_render_ops: Option<usize>,
    pub max_assign_bytes: Option<usize>,
    pub max_depth: Option<usize>,
    pub strict_variables: bool,
    pub strict_filters: bool,
    pub error_mode: ErrorMode,
}

/// Public result for configurable rendering.
#[derive(Debug, Default)]
pub struct RenderOutput {
    pub output: String,
    pub errors: Vec<Error>,
}

fn default_error_formatter(error: &Error) -> String {
    error.to_string()
}

pub struct Template {
    pub(crate) template: runtime::Template,
    pub(crate) partials: Option<sync::Arc<dyn PartialStore + Send + Sync>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &dyn crate::ObjectView) -> Result<String> {
        self.render_with_options(globals, &RenderOptions::default())
            .map(|output| output.output)
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut dyn Write, globals: &dyn crate::ObjectView) -> Result<()> {
        self.render_to_with_options(writer, globals, &RenderOptions::default())
            .map(|_| ())
    }

    /// Renders an instance of the template using configurable render options.
    pub fn render_with_options(
        &self,
        globals: &dyn crate::ObjectView,
        options: &RenderOptions,
    ) -> Result<RenderOutput> {
        const BEST_GUESS: usize = 10_000;
        let mut buffer = Vec::with_capacity(BEST_GUESS);
        let errors = self.render_to_with_options(&mut buffer, globals, options)?;

        Ok(RenderOutput {
            output: convert_buffer(buffer),
            errors,
        })
    }

    /// Renders an instance of the template into a caller-provided writer using configurable options.
    pub fn render_to_with_options(
        &self,
        writer: &mut dyn Write,
        globals: &dyn crate::ObjectView,
        options: &RenderOptions,
    ) -> Result<Vec<Error>> {
        let runtime = runtime::RuntimeBuilder::new().set_globals(globals);
        let runtime = match self.partials {
            Some(ref partials) => runtime.set_partials(partials.as_ref()),
            None => runtime,
        };
        let runtime = runtime.build();

        runtime::install_prod_policy(
            &runtime,
            runtime::ProdPolicyConfig {
                max_render_ops: options.max_render_ops,
                max_assign_bytes: options.max_assign_bytes,
                max_depth: options.max_depth,
                strict_variables: options.strict_variables,
                strict_filters: options.strict_filters,
                error_mode: error_mode_to_policy_mode(options.error_mode),
            },
        );
        Runtime::registers(&runtime)
            .get_mut::<runtime::RenderedBytesRegister>()
            .reset();
        let mut writer = CountingWriter::new(
            writer,
            Runtime::registers(&runtime),
            options.max_output_bytes,
        );
        let result = self.template.render_to(&mut writer, &runtime);

        if writer.output_limit_exceeded() {
            return Err(output_limit_error());
        }

        result?;
        Ok(runtime::take_render_errors(&runtime))
    }
}

struct CountingWriter<'a, 'r> {
    inner: &'a mut dyn Write,
    registers: &'r runtime::Registers,
    max_output_bytes: Option<usize>,
    output_limit_exceeded: bool,
}

impl<'a, 'r> CountingWriter<'a, 'r> {
    fn new(
        inner: &'a mut dyn Write,
        registers: &'r runtime::Registers,
        max_output_bytes: Option<usize>,
    ) -> Self {
        Self {
            inner,
            registers,
            max_output_bytes,
            output_limit_exceeded: false,
        }
    }

    fn output_limit_exceeded(&self) -> bool {
        self.output_limit_exceeded
    }
}

impl Write for CountingWriter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let limit = self.max_output_bytes.unwrap_or(usize::MAX);
        let already_written = self
            .registers
            .get_mut::<runtime::RenderedBytesRegister>()
            .bytes();

        if already_written >= limit {
            self.output_limit_exceeded = true;
            return Err(io::Error::other("output limit exceeded"));
        }

        let remaining = limit - already_written;
        let to_write = buf.len().min(remaining);
        let written = self.inner.write(&buf[..to_write])?;
        self.registers
            .get_mut::<runtime::RenderedBytesRegister>()
            .add(written);

        if to_write < buf.len() {
            self.output_limit_exceeded = true;
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
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

fn error_mode_to_policy_mode(mode: ErrorMode) -> runtime::ProdErrorMode {
    match mode {
        ErrorMode::Strict => runtime::ProdErrorMode::Strict,
        ErrorMode::Lenient(formatter) => runtime::ProdErrorMode::Lenient(formatter),
    }
}

fn output_limit_error() -> Error {
    Error::with_msg("Output limit exceeded")
}

#[cfg(test)]
mod test {
    use super::*;

    struct PartialWriter {
        max_per_write: usize,
        buffer: Vec<u8>,
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let written = buf.len().min(self.max_per_write);
            self.buffer.extend_from_slice(&buf[..written]);
            Ok(written)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn counting_writer_latches_output_limit_on_partial_inner_write_at_limit_boundary() {
        let registers = runtime::Registers::default();
        let mut inner = PartialWriter {
            max_per_write: 2,
            buffer: Vec::new(),
        };
        let mut writer = CountingWriter::new(&mut inner, &registers, Some(3));

        let written = writer.write(b"abcd").unwrap();

        assert_eq!(written, 2);
        assert!(writer.output_limit_exceeded());
        assert_eq!(
            registers
                .get_mut::<runtime::RenderedBytesRegister>()
                .bytes(),
            2
        );
    }
}
