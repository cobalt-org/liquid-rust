use std::io::{self, Write};
use std::rc::Rc;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::model::{Value, ValueView};
use crate::parser::{self, FilterCall, Language};
use crate::runtime::{self, Renderable, Runtime};

/// Shared late-bound filter dispatcher used by the conformance harness bridge.
pub type FallbackFilterRegistry = Rc<dyn FallbackFilterResolver>;

/// Render-time fallback filter resolver used only by the conformance harness.
pub trait FallbackFilterResolver {
    /// Whether the resolver can evaluate the requested filter.
    fn has_filter(&self, name: &str) -> bool;

    /// Evaluate a deferred filter call via the bridge-specific resolver.
    fn evaluate(
        &self,
        filter: &FilterCall,
        input: &dyn ValueView,
        runtime: &dyn Runtime,
    ) -> Result<Value>;
}

/// Ruby-backed resource-limit and error-handling callbacks for the harness path.
pub trait ConformanceCallbacks {
    /// Handle a render error and optionally provide replacement output.
    fn handle_render_error(&self, runtime: &dyn Runtime, error: Error) -> Result<Option<String>>;

    /// Track render-op usage for the current render tree.
    fn increment_render_ops(&self, amount: usize) -> Result<()>;

    /// Track assign-like writes for the current render tree.
    fn increment_assign_bytes(&self, amount: usize) -> Result<()>;

    /// Validate the current render-resource state using the cumulative bytes written so far.
    fn check_resource_limits(&self, runtime: &dyn Runtime, rendered_bytes: usize) -> Result<()>;

    /// Reset per-scope resource bookkeeping for an isolated render frame.
    fn reset_resource_limits(&self) -> Result<()>;
}

/// Hidden render configuration shared with the Ruby conformance bridge.
#[derive(Clone)]
pub struct RenderConfig {
    /// Whether missing variables should raise instead of resolving to nil.
    pub strict_variables: bool,
    /// Whether missing filters should raise instead of passing input through.
    pub strict_filters: bool,
    /// Ruby-backed callbacks for resource limits and normalized errors.
    pub callbacks: Rc<dyn ConformanceCallbacks>,
    /// Optional fallback filter resolver for late-bound Ruby filters.
    pub fallback_filters: Option<FallbackFilterRegistry>,
    /// Optional live-scope session shared across nested isolated renders.
    pub live_scope_session: Option<runtime::LiveScopeSession>,
}

/// Parse a template using the supplied language for harness-only execution.
pub fn parse(source: &str, language: Arc<Language>) -> Result<runtime::Template> {
    parser::parse(source, &language).map(runtime::Template::new)
}

/// Render a parsed template through the hidden conformance-harness pipeline.
pub fn render_to(
    template: &runtime::Template,
    writer: &mut dyn Write,
    runtime: &dyn Runtime,
    config: &RenderConfig,
) -> Result<()> {
    let previous_policy = runtime
        .registers()
        .get_mut::<runtime::ActivePolicyRegister>()
        .get();
    let previous_live_scope_session = runtime.registers().live_scope_session();
    let previous_fallback_filters = runtime
        .registers()
        .get_mut::<runtime::FallbackFilterRegistryRegister>()
        .get();

    runtime::install_conformance_policy(
        runtime,
        runtime::ConformancePolicyConfig {
            strict_variables: config.strict_variables,
            strict_filters: config.strict_filters,
            callbacks: Rc::clone(&config.callbacks),
        },
    );
    runtime
        .registers()
        .set_live_scope_session(config.live_scope_session.clone());
    runtime
        .registers()
        .get_mut::<runtime::FallbackFilterRegistryRegister>()
        .set(config.fallback_filters.as_ref().map(Rc::clone));
    let _register_guard = ConformanceRegisterGuard::new(
        runtime.registers(),
        previous_policy,
        previous_live_scope_session,
        previous_fallback_filters,
        config.live_scope_session.clone(),
    );
    runtime
        .registers()
        .get_mut::<runtime::RenderedBytesRegister>()
        .reset();

    let mut writer = CountingWriter::new(writer, runtime.registers());
    template.render_to(&mut writer, runtime)
}

struct ConformanceRegisterGuard<'r> {
    registers: &'r runtime::Registers,
    previous_policy: Option<runtime::SharedRenderPolicy>,
    previous_live_scope_session: Option<runtime::LiveScopeSession>,
    previous_fallback_filters: Option<FallbackFilterRegistry>,
    installed_live_scope_session: Option<runtime::LiveScopeSession>,
}

impl<'r> ConformanceRegisterGuard<'r> {
    fn new(
        registers: &'r runtime::Registers,
        previous_policy: Option<runtime::SharedRenderPolicy>,
        previous_live_scope_session: Option<runtime::LiveScopeSession>,
        previous_fallback_filters: Option<FallbackFilterRegistry>,
        installed_live_scope_session: Option<runtime::LiveScopeSession>,
    ) -> Self {
        Self {
            registers,
            previous_policy,
            previous_live_scope_session,
            previous_fallback_filters,
            installed_live_scope_session,
        }
    }
}

impl Drop for ConformanceRegisterGuard<'_> {
    fn drop(&mut self) {
        if let Some(session) = self.installed_live_scope_session.as_ref() {
            let should_deactivate = self
                .previous_live_scope_session
                .as_ref()
                .is_none_or(|previous| !previous.shares_identity(session));
            if should_deactivate {
                session.deactivate();
            }
        }

        self.registers
            .set_live_scope_session(self.previous_live_scope_session.clone());
        self.registers
            .get_mut::<runtime::FallbackFilterRegistryRegister>()
            .set(self.previous_fallback_filters.as_ref().map(Rc::clone));
        self.registers
            .get_mut::<runtime::ActivePolicyRegister>()
            .set(self.previous_policy.as_ref().map(Rc::clone));
    }
}

struct CountingWriter<'a, 'r> {
    inner: &'a mut dyn Write,
    registers: &'r runtime::Registers,
}

impl<'a, 'r> CountingWriter<'a, 'r> {
    fn new(inner: &'a mut dyn Write, registers: &'r runtime::Registers) -> Self {
        Self { inner, registers }
    }
}

impl Write for CountingWriter<'_, '_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.registers
            .get_mut::<runtime::RenderedBytesRegister>()
            .add(written);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod test {
    use std::fmt;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use crate::parser::{Filter, FilterArguments, FilterReflection, ParameterReflection, ParseFilter};

    use super::*;

    struct NoopCallbacks;

    impl ConformanceCallbacks for NoopCallbacks {
        fn handle_render_error(
            &self,
            _runtime: &dyn Runtime,
            error: Error,
        ) -> Result<Option<String>> {
            Err(error)
        }

        fn increment_render_ops(&self, _amount: usize) -> Result<()> {
            Ok(())
        }

        fn increment_assign_bytes(&self, _amount: usize) -> Result<()> {
            Ok(())
        }

        fn check_resource_limits(
            &self,
            _runtime: &dyn Runtime,
            _rendered_bytes: usize,
        ) -> Result<()> {
            Ok(())
        }

        fn reset_resource_limits(&self) -> Result<()> {
            Ok(())
        }
    }

    struct NamedFallbackFilterResolver(&'static str);

    impl FallbackFilterResolver for NamedFallbackFilterResolver {
        fn has_filter(&self, name: &str) -> bool {
            self.0 == name
        }

        fn evaluate(
            &self,
            _filter: &FilterCall,
            _input: &dyn ValueView,
            _runtime: &dyn Runtime,
        ) -> Result<Value> {
            unreachable!("evaluate is not used in these tests")
        }
    }

    #[derive(Clone)]
    struct CompiledFilterParser;

    impl FilterReflection for CompiledFilterParser {
        fn name(&self) -> &str {
            "override_me"
        }

        fn description(&self) -> &str {
            "tests compiled filter override dispatch"
        }

        fn positional_parameters(&self) -> &'static [ParameterReflection] {
            &[]
        }

        fn keyword_parameters(&self) -> &'static [ParameterReflection] {
            &[]
        }
    }

    impl ParseFilter for CompiledFilterParser {
        fn parse(&self, _arguments: FilterArguments) -> Result<Box<dyn Filter>> {
            Ok(Box::new(CompiledFilter))
        }

        fn reflection(&self) -> &dyn FilterReflection {
            self
        }
    }

    #[derive(Debug)]
    struct CompiledFilter;

    impl fmt::Display for CompiledFilter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("override_me")
        }
    }

    impl Filter for CompiledFilter {
        fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
            Ok(Value::scalar(format!("compiled:{}", input.render())))
        }
    }

    struct OverrideCompiledFilterResolver;

    impl FallbackFilterResolver for OverrideCompiledFilterResolver {
        fn has_filter(&self, name: &str) -> bool {
            name == "override_me"
        }

        fn evaluate(
            &self,
            _filter: &FilterCall,
            input: &dyn ValueView,
            _runtime: &dyn Runtime,
        ) -> Result<Value> {
            Ok(Value::scalar(format!("fallback:{}", input.render())))
        }
    }

    #[derive(Debug)]
    struct PanicRenderable;

    impl runtime::Renderable for PanicRenderable {
        fn render_to(&self, _writer: &mut dyn Write, _runtime: &dyn Runtime) -> Result<()> {
            panic!("boom");
        }

        fn blankness(&self) -> runtime::Blankness {
            runtime::Blankness::BlankNode
        }
    }

    #[test]
    fn render_to_restores_previous_registers_after_success() {
        let runtime = runtime::RuntimeBuilder::new().build();
        runtime::install_prod_policy(
            &runtime,
            runtime::ProdPolicyConfig {
                max_depth: Some(0),
                ..Default::default()
            },
        );

        let previous_live_scope_session = runtime::LiveScopeSession::new();
        let mut previous_scope = runtime::LiveScopeSnapshot::new();
        previous_scope.insert("marker", &Value::scalar("previous"));
        let _previous_scope_guard = previous_live_scope_session.push_root_scope(previous_scope);
        runtime
            .registers()
            .set_live_scope_session(Some(previous_live_scope_session.clone()));

        let previous_fallback_filters: FallbackFilterRegistry =
            Rc::new(NamedFallbackFilterResolver("previous"));
        runtime
            .registers()
            .get_mut::<runtime::FallbackFilterRegistryRegister>()
            .set(Some(Rc::clone(&previous_fallback_filters)));

        let installed_live_scope_session = runtime::LiveScopeSession::new();
        let installed_fallback_filters: FallbackFilterRegistry =
            Rc::new(NamedFallbackFilterResolver("installed"));
        let config = RenderConfig {
            strict_variables: true,
            strict_filters: true,
            callbacks: Rc::new(NoopCallbacks),
            fallback_filters: Some(Rc::clone(&installed_fallback_filters)),
            live_scope_session: Some(installed_live_scope_session.clone()),
        };

        render_to(
            &runtime::Template::new(Vec::new()),
            &mut Vec::new(),
            &runtime,
            &config,
        )
        .unwrap();

        let restored_live_scope_session = runtime.registers().live_scope_session().unwrap();
        assert!(restored_live_scope_session.find_root("marker").is_some());
        assert!(!installed_live_scope_session.is_active());
        assert!(runtime::enter_render_scope(&runtime).is_err());
        assert!(runtime
            .registers()
            .get_mut::<runtime::FallbackFilterRegistryRegister>()
            .get()
            .unwrap()
            .has_filter("previous"));
    }

    #[test]
    fn render_to_restores_registers_after_panic() {
        let runtime = runtime::RuntimeBuilder::new().build();
        let installed_live_scope_session = runtime::LiveScopeSession::new();
        let installed_fallback_filters: FallbackFilterRegistry =
            Rc::new(NamedFallbackFilterResolver("installed"));
        let config = RenderConfig {
            strict_variables: true,
            strict_filters: true,
            callbacks: Rc::new(NoopCallbacks),
            fallback_filters: Some(installed_fallback_filters),
            live_scope_session: Some(installed_live_scope_session.clone()),
        };

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut sink = Vec::new();
            render_to(
                &runtime::Template::new(vec![Box::new(PanicRenderable)]),
                &mut sink,
                &runtime,
                &config,
            )
        }));

        assert!(panic_result.is_err());
        assert!(runtime.registers().live_scope_session().is_none());
        assert!(!installed_live_scope_session.is_active());
        assert!(runtime
            .registers()
            .get_mut::<runtime::FallbackFilterRegistryRegister>()
            .get()
            .is_none());
        assert!(runtime
            .registers()
            .get_mut::<runtime::ActivePolicyRegister>()
            .get()
            .is_none());
    }

    #[test]
    fn render_to_uses_fallback_registry_for_compiled_filters() {
        let mut language = Language::default();
        std::sync::Arc::make_mut(&mut language.filters)
            .register("override_me".to_owned(), Box::new(CompiledFilterParser));
        let template = parse("{{ price | override_me }}", Arc::new(language)).unwrap();

        let globals = crate::model::Object::from_iter([("price".into(), Value::scalar(42))]);
        let runtime = runtime::RuntimeBuilder::new().set_globals(&globals).build();
        let mut output = Vec::new();

        render_to(
            &template,
            &mut output,
            &runtime,
            &RenderConfig {
                strict_variables: true,
                strict_filters: true,
                callbacks: Rc::new(NoopCallbacks),
                fallback_filters: Some(Rc::new(OverrideCompiledFilterResolver)),
                live_scope_session: None,
            },
        )
        .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "fallback:42");
    }
}
