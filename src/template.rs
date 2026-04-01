use std::io::Write;
use std::sync;

use liquid_core::error::Result;
use liquid_core::model::{KString, KStringCow, KStringRef, ScalarCow, Value, ValueCow};
use liquid_core::runtime;
use liquid_core::runtime::PartialStore;
use liquid_core::runtime::Renderable;

pub struct Template {
    pub(crate) template: runtime::Template,
    pub(crate) partials: Option<sync::Arc<dyn PartialStore + Send + Sync>>,
}

impl Template {
    /// Renders an instance of the Template, using the given globals.
    pub fn render(&self, globals: &dyn crate::ObjectView) -> Result<String> {
        const BEST_GUESS: usize = 10_000;
        let mut data = Vec::with_capacity(BEST_GUESS);
        self.render_to(&mut data, globals)?;

        Ok(convert_buffer(data))
    }

    /// Renders an instance of the Template, using the given globals.
    pub fn render_to(&self, writer: &mut dyn Write, globals: &dyn crate::ObjectView) -> Result<()> {
        let runtime = runtime::RuntimeBuilder::new().set_globals(globals);
        let runtime = match self.partials {
            Some(ref partials) => runtime.set_partials(partials.as_ref()),
            None => runtime,
        };
        let runtime = runtime.build();
        self.template.render_to(writer, &runtime)
    }

    /// Renders an instance of the Template with a caller-provided runtime.
    pub fn render_to_runtime(
        &self,
        writer: &mut dyn Write,
        runtime: &dyn runtime::Runtime,
    ) -> Result<()> {
        let runtime = TemplateRuntime {
            inner: runtime,
            partials: self.partials.as_deref(),
        };
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

struct TemplateRuntime<'a> {
    inner: &'a dyn runtime::Runtime,
    partials: Option<&'a (dyn PartialStore + Send + Sync)>,
}

impl runtime::Runtime for TemplateRuntime<'_> {
    fn partials(&self) -> &dyn PartialStore {
        if let Some(partials) = self.partials {
            partials
        } else {
            self.inner.partials()
        }
    }

    fn name(&self) -> Option<KStringRef<'_>> {
        self.inner.name()
    }

    fn roots(&self) -> std::collections::BTreeSet<KStringCow<'_>> {
        self.inner.roots()
    }

    fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
        self.inner.try_get(path)
    }

    fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
        self.inner.get(path)
    }

    fn set_global(&self, name: KString, val: Value) -> Option<Value> {
        self.inner.set_global(name, val)
    }

    fn set_index(&self, name: KString, val: Value) -> Option<Value> {
        self.inner.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
        self.inner.get_index(name)
    }

    fn registers(&self) -> &runtime::Registers {
        self.inner.registers()
    }

    fn evaluate_filter(
        &self,
        filter: &liquid_core::parser::FilterCall,
        input: &dyn crate::ValueView,
        fallback_filters: &liquid_core::parser::PluginRegistry<Box<dyn liquid_core::parser::ParseFilter>>,
    ) -> Result<Value> {
        self.inner.evaluate_filter(filter, input, fallback_filters)
    }

    fn handle_render_error(&self, error: liquid_core::Error) -> Result<Option<String>> {
        self.inner.handle_render_error(error)
    }
}

#[cfg(test)]
mod tests {
    use liquid_core::parser::{FilterCall, ParseFilter, PluginRegistry};
    use liquid_core::runtime::{Runtime, RuntimeBuilder};

    use super::*;

    struct RecoveringRuntime<'a> {
        inner: &'a dyn Runtime,
    }

    impl Runtime for RecoveringRuntime<'_> {
        fn partials(&self) -> &dyn PartialStore {
            self.inner.partials()
        }

        fn name(&self) -> Option<KStringRef<'_>> {
            self.inner.name()
        }

        fn roots(&self) -> std::collections::BTreeSet<KStringCow<'_>> {
            self.inner.roots()
        }

        fn try_get(&self, path: &[ScalarCow<'_>]) -> Option<ValueCow<'_>> {
            self.inner.try_get(path)
        }

        fn get(&self, path: &[ScalarCow<'_>]) -> Result<ValueCow<'_>> {
            self.inner.get(path)
        }

        fn set_global(&self, name: KString, val: Value) -> Option<Value> {
            self.inner.set_global(name, val)
        }

        fn set_index(&self, name: KString, val: Value) -> Option<Value> {
            self.inner.set_index(name, val)
        }

        fn get_index<'a>(&'a self, name: &str) -> Option<ValueCow<'a>> {
            self.inner.get_index(name)
        }

        fn registers(&self) -> &runtime::Registers {
            self.inner.registers()
        }

        fn evaluate_filter(
            &self,
            filter: &FilterCall,
            input: &dyn crate::ValueView,
            fallback_filters: &PluginRegistry<Box<dyn ParseFilter>>,
        ) -> Result<Value> {
            self.inner.evaluate_filter(filter, input, fallback_filters)
        }

        fn handle_render_error(&self, _error: liquid_core::Error) -> Result<Option<String>> {
            Ok(Some("ERR".to_string()))
        }
    }

    #[test]
    fn render_to_runtime_continues_after_recovered_error() {
        let parser = crate::ParserBuilder::with_stdlib().build().unwrap();
        let template = parser.parse("A{{ 1 | divided_by: 0 }}B").unwrap();
        let globals = crate::Object::new();
        let base_runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let runtime = RecoveringRuntime {
            inner: &base_runtime,
        };
        let mut output = Vec::new();

        template.render_to_runtime(&mut output, &runtime).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "AERRB");
    }
}
