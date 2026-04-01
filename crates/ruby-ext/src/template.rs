use std::borrow::Cow;
use std::cell::RefCell;
use std::error::Error as StdError;
use std::fmt;

use liquid::model::{DisplayCow, KStringCow, State, Value as LiquidValue};
use liquid::partials::{OnDemandCompiler, PartialSource};
use liquid::ObjectView;
use liquid::ParserBuilder;
use liquid::Template as LiquidTemplate;
use liquid::ValueView;
use liquid_core::parser::{FilterCall, ParseFilter, PluginRegistry};
use liquid_core::runtime::evaluate_filter_with_registry;
use liquid_core::Runtime;
use magnus::{
    class::Class,
    r_array::RArray,
    r_hash::RHash,
    typed_data,
    value::{InnerValue, Opaque, ReprValue},
    Error as MagnusError, Exception, ExceptionClass, IntoValue, Module, RClass, RModule,
    TryConvert, Value,
};

use crate::errors;
use crate::values;

#[magnus::wrap(
    class = "Liquid::RustExtension::NativeTemplate",
    free_immediately,
    size
)]
struct NativeTemplate {
    template: LiquidTemplate,
}

#[derive(Clone, Copy)]
struct HostPartialSource {
    file_system: Opaque<Value>,
}

impl fmt::Debug for HostPartialSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HostPartialSource")
    }
}

impl PartialSource for HostPartialSource {
    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn get<'a>(&'a self, name: &str) -> liquid_core::Result<Option<Cow<'a, str>>> {
        match load_host_partial(self.file_system, name) {
            Ok(source) => Ok(Some(Cow::Owned(source))),
            Err(error) => Err(error),
        }
    }
}

#[derive(Clone)]
struct HostPartialLoadError {
    class_name: String,
    message: String,
    original: Option<Opaque<Value>>,
}

impl fmt::Debug for HostPartialLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostPartialLoadError")
            .field("class_name", &self.class_name)
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for HostPartialLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.class_name, self.message)
    }
}

impl StdError for HostPartialLoadError {}

impl HostPartialLoadError {
    fn from_host_error(error: &MagnusError) -> Self {
        let message = error.to_string();
        let Some(value) = error.value() else {
            return Self {
                class_name: "RuntimeError".to_owned(),
                message,
                original: None,
            };
        };

        let class_name = unsafe { value.classname() }.to_string();
        let message = value
            .funcall::<_, _, String>("message", ())
            .unwrap_or(message);
        let message = normalize_host_error_message(&class_name, message);
        Self {
            class_name,
            message,
            original: Some(Opaque::from(value)),
        }
    }
}

fn normalize_host_error_message(class_name: &str, message: String) -> String {
    if !class_name.starts_with("Liquid::") {
        return message;
    }

    if let Some(stripped) = message.strip_prefix("Liquid error: ") {
        return stripped.to_owned();
    }

    if let Some(stripped) = message.strip_prefix("Liquid syntax error: ") {
        return stripped.to_owned();
    }

    message
}

pub(crate) fn ext_parse(
    ruby: &magnus::Ruby,
    source: String,
    line_numbers: bool,
    error_mode: Option<String>,
    environment: Option<RHash>,
) -> Result<RHash, MagnusError> {
    let template = parse_template_with_environment(ruby, &source, environment)?;
    let handle = ruby.hash_new();
    handle.aset("source", source)?;
    handle.aset("line_numbers", line_numbers)?;
    handle.aset(
        "error_mode",
        error_mode.unwrap_or_else(|| "strict".to_string()),
    )?;
    handle.aset(
        "environment",
        environment.map_or_else(|| ruby.qnil().as_value(), ReprValue::as_value),
    )?;
    handle.aset("errors", ruby.ary_new())?;
    handle.aset("warnings", ruby.ary_new())?;
    handle.aset("root", build_root_handle(ruby, &template)?)?;
    handle.aset(
        "template",
        ruby.obj_wrap(NativeTemplate { template })
            .into_value_with(ruby),
    )?;
    Ok(handle)
}

pub(crate) fn ext_render(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
) -> Result<String, MagnusError> {
    render_internal(ruby, handle, context_or_assigns, false)
}

pub(crate) fn ext_render_strict(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
) -> Result<String, MagnusError> {
    render_internal(ruby, handle, context_or_assigns, true)
}

pub(crate) fn ext_template_root(_ruby: &magnus::Ruby, handle: RHash) -> Result<Value, MagnusError> {
    handle.lookup("root")
}

pub(crate) fn ext_template_errors(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    handle.lookup("errors")
}

pub(crate) fn ext_template_warnings(
    _ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    handle.lookup("warnings")
}

pub(crate) fn ext_debug_payload(
    _ruby: &magnus::Ruby,
    context_or_assigns: Value,
) -> Result<Vec<String>, MagnusError> {
    let globals = globals_from_context(context_or_assigns, values::LookupMode::Materialized)?;
    Ok(globals.keys().map(|key| key.to_string()).collect())
}

fn render_internal(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
    strict: bool,
) -> Result<String, MagnusError> {
    let errors: RArray = handle.lookup("errors")?;
    errors.clear()?;
    let warnings: RArray = handle.lookup("warnings")?;
    warnings.clear()?;

    let filter_host = build_filter_host(ruby, handle, context_or_assigns)?;
    let template = lookup_template(handle)?;
    let lookup_mode = if strict_variables_enabled(context_or_assigns) && !strict {
        values::LookupMode::TrackMissing
    } else {
        values::LookupMode::Materialized
    };
    let globals = globals_from_context(context_or_assigns, lookup_mode)?;
    let strict_variables = strict_variables_enabled(context_or_assigns);
    let lenient_globals = LenientObject::new(&globals as &dyn ObjectView);
    let render_globals: &dyn ObjectView = if strict_variables && strict {
        &globals
    } else {
        &lenient_globals
    };

    let base_runtime = liquid_core::runtime::RuntimeBuilder::new()
        .set_globals(render_globals)
        .build();
    let runtime = DynamicFilterRuntime::new(&base_runtime, filter_host, context_or_assigns, strict);
    let mut rendered = Vec::new();
    let render_result = template.template.render_to_runtime(&mut rendered, &runtime);
    let rendered = String::from_utf8(rendered).expect("render should stay valid utf-8");
    for handled_error in runtime.handled_errors(ruby) {
        errors.push(handled_error)?;
    }

    if let Some(raised_exception) = runtime.take_raised_exception() {
        return Err(raised_exception);
    }

    match render_result {
        Ok(()) => {
            let tracked_errors = globals.take_errors();
            if let Some(message) = tracked_errors.first() {
                for error in &tracked_errors {
                    errors.push(error.clone())?;
                }
                if strict {
                    return Err(MagnusError::new(
                        ruby.exception_runtime_error(),
                        message.clone(),
                    ));
                }
            }

            Ok(rendered)
        }
        Err(error) => {
            if let Some(load_error) = host_partial_load_error(&error) {
                return Err(rebuild_host_error(ruby, load_error));
            }

            let message = error.to_string();
            let tracked_errors = globals.take_errors();
            if strict
                && (message.contains("Unknown variable") || message.contains("Unknown index"))
                && !tracked_errors.is_empty()
            {
                for tracked_error in &tracked_errors {
                    errors.push(tracked_error.clone())?;
                }
                return Err(MagnusError::new(
                    ruby.exception_runtime_error(),
                    tracked_errors[0].clone(),
                ));
            }

            errors.push(message.clone())?;
            for tracked_error in tracked_errors {
                errors.push(tracked_error)?;
            }
            if strict {
                Err(errors::runtime_error(ruby, message))
            } else if is_memory_limit_error(&message) {
                Ok(non_strict_error_output(&message))
            } else {
                Ok(rendered)
            }
        }
    }
}

fn parse_template(ruby: &magnus::Ruby, source: &str) -> Result<LiquidTemplate, MagnusError> {
    parse_template_with_environment(ruby, source, None)
}

fn parse_template_with_environment(
    ruby: &magnus::Ruby,
    source: &str,
    environment: Option<RHash>,
) -> Result<LiquidTemplate, MagnusError> {
    let parser = if let Some(file_system) = environment
        .and_then(|handle| handle.get("file_system"))
        .filter(|value| !value.is_nil())
    {
        ParserBuilder::with_stdlib()
            .partials(OnDemandCompiler::new(HostPartialSource {
                file_system: Opaque::from(file_system),
            }))
            .build()
    } else {
        ParserBuilder::with_stdlib().build()
    }
    .map_err(|error| errors::runtime_error(ruby, error.to_string()))?;

    parser
        .parse(source)
        .map_err(|error| errors::syntax_error(ruby, error.to_string()))
}

fn load_host_partial(file_system: Opaque<Value>, name: &str) -> liquid_core::Result<String> {
    let vm = magnus::Ruby::get().expect("VM should be available");
    let file_system = file_system.get_inner_with(&vm);

    let value = file_system
        .funcall::<_, _, Value>("read_template_file", (name,))
        .map_err(|error| {
            liquid_core::Error::with_msg("Host partial load failed")
                .context("requested partial", name.to_owned())
                .cause(HostPartialLoadError::from_host_error(&error))
        })?;

    Ok(String::try_convert(value)
        .ok()
        .unwrap_or_else(|| value.to_string()))
}

fn host_partial_load_error(error: &liquid_core::Error) -> Option<&HostPartialLoadError> {
    error
        .source()
        .and_then(|cause| cause.downcast_ref::<HostPartialLoadError>())
}

fn rebuild_host_error(vm: &magnus::Ruby, error: &HostPartialLoadError) -> MagnusError {
    if let Some(original) = error.original {
        if let Some(exception) = Exception::from_value(original.get_inner_with(vm)) {
            return MagnusError::from(exception);
        }
    }

    match lookup_exception_class(vm, &error.class_name) {
        Ok(class) => MagnusError::new(class, error.message.clone()),
        Err(_) => MagnusError::new(vm.exception_runtime_error(), error.message.clone()),
    }
}

fn lookup_exception_class(
    vm: &magnus::Ruby,
    class_name: &str,
) -> Result<ExceptionClass, MagnusError> {
    let mut current = vm.class_object().as_value();
    for segment in class_name.split("::").filter(|segment| !segment.is_empty()) {
        current = current.funcall("const_get", (segment,))?;
    }

    ExceptionClass::from_value(current)
        .ok_or_else(|| MagnusError::new(vm.exception_type_error(), class_name.to_owned()))
}

fn lookup_template(handle: RHash) -> Result<typed_data::Obj<NativeTemplate>, MagnusError> {
    handle.lookup("template")
}

fn build_root_handle(
    ruby: &magnus::Ruby,
    _template: &LiquidTemplate,
) -> Result<RArray, MagnusError> {
    let nodes = ruby.ary_new();
    Ok(nodes)
}

fn globals_from_context(
    context_or_assigns: Value,
    lookup_mode: values::LookupMode,
) -> Result<values::RenderRootObject, MagnusError> {
    let context = current_context(context_or_assigns);

    if let Ok(payload) = String::try_convert(context_or_assigns) {
        return values::json_to_object(&payload).map(values::RenderRootObject::from_liquid_object);
    }

    if let Some(handle) = RHash::from_value(context_or_assigns) {
        if let Some(scopes) = handle.get("scopes").and_then(RArray::from_value) {
            let mut values = Vec::with_capacity(scopes.len());
            for idx in 0..scopes.len() {
                let scope: Value = scopes.entry(idx as isize)?;
                values.push(scope);
            }
            return values::RenderRootObject::from_values_with_mode_and_context(
                values,
                lookup_mode,
                context,
            );
        }

        return values::RenderRootObject::from_value_with_mode_and_context(
            handle.as_value(),
            lookup_mode,
            context,
        );
    }

    values::RenderRootObject::from_value_with_mode_and_context(
        context_or_assigns,
        lookup_mode,
        context,
    )
}

fn current_context(context_or_assigns: Value) -> Option<Value> {
    RHash::from_value(context_or_assigns).and_then(|handle| handle.get("context"))
}

fn strict_variables_enabled(context_or_assigns: Value) -> bool {
    RHash::from_value(context_or_assigns)
        .and_then(|handle| handle.get("strict_variables"))
        .and_then(|value| bool::try_convert(value).ok())
        .unwrap_or(false)
}

fn build_filter_host(
    ruby: &magnus::Ruby,
    handle: RHash,
    context_or_assigns: Value,
) -> Result<Option<Value>, MagnusError> {
    let mut modules = Vec::new();

    if let Some(environment) = handle.get("environment").and_then(RHash::from_value) {
        collect_filter_modules(environment, &mut modules)?;
    }

    if let Some(context) = RHash::from_value(context_or_assigns) {
        collect_filter_modules(context, &mut modules)?;
    }

    if modules.is_empty() {
        return Ok(None);
    }

    let host = ruby.class_object().new_instance(())?.as_value();
    for module in modules {
        host.funcall::<_, _, Value>("extend", (module,))?;
    }

    Ok(Some(host))
}

fn collect_filter_modules(handle: RHash, modules: &mut Vec<RModule>) -> Result<(), MagnusError> {
    let Some(filters) = handle.get("filters").and_then(RArray::from_value) else {
        return Ok(());
    };

    for idx in 0..filters.len() {
        let module: RModule = filters.entry(idx as isize)?;
        modules.push(module);
    }

    Ok(())
}

struct DynamicFilterRuntime<'a> {
    inner: &'a dyn Runtime,
    filter_host: Option<Value>,
    recovery: Option<RenderRecoveryState>,
    persistent_assigns: Option<RHash>,
    resource_limits: Option<Value>,
}

impl<'a> DynamicFilterRuntime<'a> {
    fn new(
        inner: &'a dyn Runtime,
        filter_host: Option<Value>,
        context_or_assigns: Value,
        strict: bool,
    ) -> Self {
        Self {
            inner,
            filter_host,
            recovery: (!strict).then(|| RenderRecoveryState::new(context_or_assigns)),
            persistent_assigns: persistent_assigns_from_context(context_or_assigns),
            resource_limits: resource_limits_from_context(context_or_assigns),
        }
    }

    fn handled_errors(&self, ruby: &magnus::Ruby) -> Vec<Value> {
        self.recovery
            .as_ref()
            .map(|recovery| {
                recovery
                    .handled_errors
                    .borrow()
                    .iter()
                    .map(|error| error.to_value(ruby))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn take_raised_exception(&self) -> Option<MagnusError> {
        self.recovery
            .as_ref()
            .and_then(|recovery| recovery.raised_exception.borrow_mut().take())
    }

    fn try_host_filter(
        &self,
        filter: &FilterCall,
        input: &dyn ValueView,
    ) -> liquid_core::Result<Option<LiquidValue>> {
        let Some(host) = self.filter_host else {
            return Ok(None);
        };

        if !host.respond_to(filter.name(), false).unwrap_or(false) {
            return Ok(None);
        }

        let vm = magnus::Ruby::get().expect("VM should be available");
        let mut args = Vec::new();
        args.push(
            values::liquid_to_ruby_value(&vm, &input.to_value())
                .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?,
        );

        let filter_args = filter.args();
        for expression in filter_args.positional {
            let value = expression.evaluate(self.inner)?;
            args.push(
                values::liquid_to_ruby_value(&vm, &value.to_value())
                    .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?,
            );
        }

        let mut kwargs = Vec::new();
        for (name, expression) in filter_args.keyword {
            kwargs.push((name, expression.evaluate(self.inner)?.to_value()));
        }

        if !kwargs.is_empty() {
            let keyword_hash = vm.hash_new();
            for (name, value) in kwargs {
                keyword_hash
                    .aset(
                        name,
                        values::liquid_to_ruby_value(&vm, &value)
                            .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?,
                    )
                    .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?;
            }
            args.push(keyword_hash.into_value_with(&vm));
        }

        let mut send_args = Vec::with_capacity(args.len() + 1);
        send_args.push(vm.to_symbol(filter.name()).as_value());
        send_args.extend(args);

        let result: Value = host
            .funcall("public_send", &send_args[..])
            .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?;

        values::ruby_to_liquid_value(result)
            .map(Some)
            .map_err(|error| liquid_core::Error::with_msg(error.to_string()))
    }
}

impl Runtime for DynamicFilterRuntime<'_> {
    fn partials(&self) -> &dyn liquid_core::runtime::PartialStore {
        self.inner.partials()
    }

    fn name(&self) -> Option<liquid::model::KStringRef<'_>> {
        self.inner.name()
    }

    fn roots(&self) -> std::collections::BTreeSet<KStringCow<'_>> {
        self.inner.roots()
    }

    fn try_get(
        &self,
        path: &[liquid::model::ScalarCow<'_>],
    ) -> Option<liquid::model::ValueCow<'_>> {
        self.inner.try_get(path)
    }

    fn get(
        &self,
        path: &[liquid::model::ScalarCow<'_>],
    ) -> liquid_core::Result<liquid::model::ValueCow<'_>> {
        self.inner.get(path)
    }

    fn set_global(
        &self,
        name: liquid::model::KString,
        val: liquid::model::Value,
    ) -> Option<liquid::model::Value> {
        if let Some(target) = self.persistent_assigns {
            if let Ok(vm) = magnus::Ruby::get() {
                if let Ok(value) = values::liquid_to_ruby_value(&vm, &val) {
                    let _ = target.aset(name.as_str(), value);
                }
            }
        }

        self.inner.set_global(name, val)
    }

    fn set_index(
        &self,
        name: liquid::model::KString,
        val: liquid::model::Value,
    ) -> Option<liquid::model::Value> {
        self.inner.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<liquid::model::ValueCow<'a>> {
        self.inner.get_index(name)
    }

    fn registers(&self) -> &liquid_core::runtime::Registers {
        self.inner.registers()
    }

    fn evaluate_filter(
        &self,
        filter: &FilterCall,
        input: &dyn ValueView,
        fallback_filters: &PluginRegistry<Box<dyn ParseFilter>>,
    ) -> liquid_core::Result<LiquidValue> {
        if let Some(value) = self.try_host_filter(filter, input)? {
            return Ok(value);
        }

        evaluate_filter_with_registry(self.inner, filter, input, fallback_filters)
    }

    fn handle_render_error(
        &self,
        error: liquid_core::Error,
    ) -> liquid_core::Result<Option<String>> {
        if let Some(load_error) = host_partial_load_error(&error) {
            let Some(recovery) = &self.recovery else {
                return Err(error);
            };

            if recovery.raised_exception.borrow().is_some() {
                return Err(error);
            }

            let vm = magnus::Ruby::get().expect("VM should be available");
            let message = render_default_error_output(&load_error.message);
            let error_value = build_host_exception_value(&vm, load_error).ok();
            if let Some(error_value) = error_value {
                recovery
                    .handled_errors
                    .borrow_mut()
                    .push(HandledRenderError::Value(Opaque::from(error_value)));

                if let Some(renderer) = recovery.exception_renderer {
                    return match call_exception_renderer_with_value(renderer, error_value) {
                        Ok(replacement) => Ok(Some(replacement)),
                        Err(error) => {
                            recovery
                                .raised_exception
                                .borrow_mut()
                                .replace(wrap_exception_renderer_error(error));
                            Err(liquid_core::Error::with_msg("exception renderer raised"))
                        }
                    };
                }
            } else {
                recovery
                    .handled_errors
                    .borrow_mut()
                    .push(HandledRenderError::Message(message.clone()));

                if let Some(renderer) = recovery.exception_renderer {
                    return match call_exception_renderer(renderer, &message) {
                        Ok(replacement) => Ok(Some(replacement)),
                        Err(error) => {
                            recovery
                                .raised_exception
                                .borrow_mut()
                                .replace(wrap_exception_renderer_error(error));
                            Err(liquid_core::Error::with_msg("exception renderer raised"))
                        }
                    };
                }
            }

            return Ok(Some(message));
        }

        if is_memory_limit_error(&error.to_string()) {
            return Err(error);
        }

        let Some(recovery) = &self.recovery else {
            return Err(error);
        };

        if recovery.raised_exception.borrow().is_some() {
            return Err(error);
        }

        let message = error.to_string();
        recovery
            .handled_errors
            .borrow_mut()
            .push(HandledRenderError::Message(message.clone()));

        if preserve_partial_output(&message) {
            return Ok(None);
        }

        if let Some(renderer) = recovery.exception_renderer {
            return match call_exception_renderer(renderer, &message) {
                Ok(replacement) => Ok(Some(replacement)),
                Err(error) => {
                    recovery
                        .raised_exception
                        .borrow_mut()
                        .replace(wrap_exception_renderer_error(error));
                    Err(liquid_core::Error::with_msg("exception renderer raised"))
                }
            };
        }

        Ok(Some(render_default_error_output(&message)))
    }

    fn increment_render_score(&self, amount: usize) -> liquid_core::Result<()> {
        call_resource_limits_method(self.resource_limits, "increment_render_score", amount)
    }

    fn increment_assign_score(&self, amount: usize) -> liquid_core::Result<()> {
        call_resource_limits_method(self.resource_limits, "increment_assign_score", amount)
    }

    fn check_resource_limits(&self) -> liquid_core::Result<()> {
        let current_bytes = self
            .registers()
            .get_mut::<liquid_core::runtime::RenderedBytesRegister>()
            .bytes();
        call_resource_limits_method(self.resource_limits, "increment_write_score", current_bytes)
    }

    fn reset_resource_limits(&self) -> liquid_core::Result<()> {
        call_resource_limits_reset(self.resource_limits)
    }
}

struct RenderRecoveryState {
    exception_renderer: Option<Value>,
    handled_errors: RefCell<Vec<HandledRenderError>>,
    raised_exception: RefCell<Option<MagnusError>>,
}

impl RenderRecoveryState {
    fn new(context_or_assigns: Value) -> Self {
        let exception_renderer = RHash::from_value(context_or_assigns)
            .and_then(|handle| handle.get("exception_renderer"))
            .filter(|value| !value.is_nil());

        Self {
            exception_renderer,
            handled_errors: RefCell::new(Vec::new()),
            raised_exception: RefCell::new(None),
        }
    }
}

enum HandledRenderError {
    Message(String),
    Value(Opaque<Value>),
}

impl HandledRenderError {
    fn to_value(&self, ruby: &magnus::Ruby) -> Value {
        match self {
            Self::Message(message) => ruby.str_new(message).as_value(),
            Self::Value(value) => value.get_inner_with(ruby),
        }
    }
}

fn persistent_assigns_from_context(context_or_assigns: Value) -> Option<RHash> {
    if let Some(handle) = RHash::from_value(context_or_assigns) {
        if let Some(persistent_assigns) =
            handle.get("persistent_assigns").and_then(RHash::from_value)
        {
            return Some(persistent_assigns);
        }

        if let Some(scopes) = handle.get("scopes").and_then(RArray::from_value) {
            for idx in (0..scopes.len()).rev() {
                let scope: Value = scopes.entry(idx as isize).ok()?;
                if let Some(hash) = RHash::from_value(scope) {
                    return Some(hash);
                }
            }
            return None;
        }

        return Some(handle);
    }

    RHash::from_value(context_or_assigns)
}

fn resource_limits_from_context(context_or_assigns: Value) -> Option<Value> {
    current_context(context_or_assigns).and_then(|context| {
        context
            .funcall::<_, _, Value>("resource_limits", ())
            .ok()
            .filter(|value| !value.is_nil())
    })
}

fn call_resource_limits_method(
    resource_limits: Option<Value>,
    method: &str,
    amount: usize,
) -> liquid_core::Result<()> {
    let Some(resource_limits) = resource_limits else {
        return Ok(());
    };

    resource_limits
        .funcall::<_, _, Value>(method, (amount as i64,))
        .map(|_| ())
        .map_err(|error| liquid_core::Error::with_msg(error.to_string()))
}

fn call_resource_limits_reset(resource_limits: Option<Value>) -> liquid_core::Result<()> {
    let Some(resource_limits) = resource_limits else {
        return Ok(());
    };

    resource_limits
        .funcall::<_, _, Value>("reset", ())
        .map(|_| ())
        .map_err(|error| liquid_core::Error::with_msg(error.to_string()))
}

fn is_memory_limit_error(message: &str) -> bool {
    message.contains("Memory limits exceeded")
}

fn non_strict_error_output(message: &str) -> String {
    if is_memory_limit_error(message) {
        "Liquid error: Memory limits exceeded".to_string()
    } else if message.starts_with("Liquid error: ") {
        message.to_string()
    } else {
        render_default_error_output(message)
    }
}

struct RenderErrorMetadata {
    message: String,
    line_number: Option<usize>,
}

impl RenderErrorMetadata {
    fn from_raw(message: &str) -> Self {
        let line_number = extract_line_number(message);

        if message.contains("Unknown filter") {
            let requested = extract_context_value(message, "requested filter=");
            return Self {
                message: requested
                    .map(|requested| format!("undefined filter {requested}"))
                    .unwrap_or_else(|| "undefined filter".to_string()),
                line_number,
            };
        }

        if message.contains("Can't divide by zero") {
            return Self {
                message: "divided by 0".to_string(),
                line_number,
            };
        }

        if message.contains("Undefined drop method") {
            let requested = extract_context_value(message, "requested variable=");
            return Self {
                message: requested
                    .map(|requested| format!("undefined drop method {requested}"))
                    .unwrap_or_else(|| "undefined drop method".to_string()),
                line_number,
            };
        }

        if message.contains("Unknown variable") {
            let requested = extract_context_value(message, "requested variable=");
            return Self {
                message: requested
                    .map(|requested| format!("undefined variable {requested}"))
                    .unwrap_or_else(|| "undefined variable".to_string()),
                line_number,
            };
        }

        Self {
            message: normalize_error_message(message),
            line_number,
        }
    }

    fn render(&self) -> String {
        let mut rendered = String::from("Liquid error");
        if let Some(line_number) = self.line_number {
            rendered.push_str(&format!(" (line {line_number})"));
        }
        rendered.push_str(": ");
        rendered.push_str(&self.message);
        rendered
    }
}

fn preserve_partial_output(message: &str) -> bool {
    message.contains("Unknown filter")
        || message.contains("Unknown variable")
        || message.contains("Undefined drop method")
}

fn render_default_error_output(message: &str) -> String {
    RenderErrorMetadata::from_raw(message).render()
}

fn extract_context_value(message: &str, key: &str) -> Option<String> {
    message.lines().find_map(|line| {
        line.split_once(key)
            .map(|(_, value)| value.trim().to_string())
    })
}

fn extract_line_number(message: &str) -> Option<usize> {
    message.lines().find_map(|line| {
        let (_, remainder) = line.split_once("-->")?;
        let remainder = remainder.trim_start();
        let digits: String = remainder
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect();
        if digits.is_empty() {
            None
        } else {
            digits.parse().ok()
        }
    })
}

fn normalize_error_message(message: &str) -> String {
    if message.contains("Unknown tag.") {
        let requested = extract_context_value(message, "requested=");
        return match requested.as_deref() {
            Some("else") => "Unexpected outer 'else' tag".to_string(),
            Some(requested) => format!("Unknown tag '{requested}'"),
            None => "Unknown tag".to_string(),
        };
    }

    if message.contains("expected Identifier or Value") {
        return "is not a valid expression".to_string();
    }

    if let Some(line) = message
        .lines()
        .find(|line| line.trim_start().starts_with('='))
    {
        return line
            .split_once('=')
            .map(|(_, value)| value.trim().to_string())
            .unwrap_or_else(|| line.trim().to_string());
    }

    message
        .strip_prefix("liquid: ")
        .unwrap_or(message)
        .trim()
        .to_string()
}

fn call_exception_renderer(renderer: Value, message: &str) -> Result<String, MagnusError> {
    let wrapped_error = wrap_liquid_error(message)?;
    call_exception_renderer_with_value(renderer, wrapped_error)
}

fn call_exception_renderer_with_value(
    renderer: Value,
    error: Value,
) -> Result<String, MagnusError> {
    let rendered: Value = renderer.funcall("call", (error,))?;
    rendered.funcall("to_s", ())
}

fn build_host_exception_value(
    vm: &magnus::Ruby,
    error: &HostPartialLoadError,
) -> Result<Value, MagnusError> {
    if let Some(original) = error.original {
        return Ok(original.get_inner_with(vm));
    }

    match lookup_exception_class(vm, &error.class_name) {
        Ok(class) => class
            .new_instance((error.message.clone(),))
            .map(ReprValue::as_value),
        Err(_) => vm
            .exception_runtime_error()
            .new_instance((error.message.clone(),))
            .map(ReprValue::as_value),
    }
}

fn wrap_liquid_error(message: &str) -> Result<Value, MagnusError> {
    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let liquid: RModule = ruby.class_object().const_get("Liquid")?;
    let liquid_error: RClass = liquid.const_get("Error")?;
    let runtime_error = ruby.exception_runtime_error().new_instance((message,))?;
    liquid_error.funcall("wrap", (runtime_error,))
}

fn wrap_exception_renderer_error(error: MagnusError) -> MagnusError {
    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let Ok(liquid) = ruby.class_object().const_get::<_, RModule>("Liquid") else {
        return error;
    };
    let Ok(template) = liquid.const_get::<_, RClass>("Template") else {
        return error;
    };
    let Ok(wrapper) = template.const_get::<_, RClass>("ExceptionRendererRaised") else {
        return error;
    };
    if error.is_kind_of(wrapper) {
        return error;
    }
    let Some(raised) = error.value() else {
        return error;
    };
    match wrapper.new_instance((raised,)) {
        Ok(exception) => Exception::from_value(exception)
            .map(MagnusError::from)
            .unwrap_or(error),
        Err(_) => error,
    }
}

static NIL_VALUE: LiquidValue = LiquidValue::Nil;

struct LenientObject<'a> {
    inner: &'a dyn ObjectView,
}

impl<'a> LenientObject<'a> {
    fn new(inner: &'a dyn ObjectView) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for LenientObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl ValueView for LenientObject<'_> {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self.inner.as_debug()
    }

    fn render(&self) -> DisplayCow<'_> {
        self.inner.render()
    }

    fn source(&self) -> DisplayCow<'_> {
        self.inner.source()
    }

    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }

    fn query_state(&self, state: State) -> bool {
        self.inner.query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        self.inner.to_kstr()
    }

    fn to_value(&self) -> LiquidValue {
        self.inner.to_value()
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for LenientObject<'_> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.inner.size()
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(
            self.inner
                .keys()
                .map(|key| KStringCow::from_string(key.into_owned().to_string())),
        )
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        self.inner.values()
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        Box::new(
            self.inner
                .iter()
                .map(|(key, value)| (KStringCow::from_string(key.into_owned().to_string()), value)),
        )
    }

    fn contains_key(&self, _index: &str) -> bool {
        true
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        self.inner
            .get(index)
            .map(|value| value as &dyn ValueView)
            .or(Some(&NIL_VALUE as &dyn ValueView))
    }
}
