use std::borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use liquid::conformance::{self, ConformanceCallbacks, FallbackFilterResolver};
use liquid::model::{DisplayCow, KStringCow, State, Value as LiquidValue};
use liquid::ObjectView;
use liquid::ParserBuilder;
use liquid::ValueView;
use liquid_core::parser::FilterCall;
use liquid_core::{parser, Runtime};
use liquid_lib::stdlib;
use magnus::{
    class::Class,
    r_array::RArray,
    r_hash::RHash,
    typed_data,
    value::{InnerValue, Opaque, ReprValue},
    Error as MagnusError, Exception, ExceptionClass, IntoValue, Module, RClass, RModule,
    TryConvert, Value,
};

use crate::context;
use crate::errors;
use crate::values;

#[magnus::wrap(
    class = "Liquid::RustExtension::NativeTemplate",
    free_immediately,
    size
)]
struct NativeTemplate {
    template: liquid_core::runtime::Template,
    partials: Option<Arc<dyn liquid_core::runtime::PartialStore + Send + Sync>>,
}

#[derive(Clone)]
struct ParseRuntimeOptions {
    line_numbers: bool,
    error_mode: String,
}

impl ParseRuntimeOptions {
    fn new(line_numbers: bool, error_mode: Option<String>) -> Self {
        Self {
            line_numbers,
            error_mode: error_mode.unwrap_or_else(|| "strict".to_string()),
        }
    }
}

struct ActivePartialState {
    file_system: Opaque<Value>,
    source_cache: HashMap<String, String>,
    cache: HashMap<String, Arc<dyn liquid_core::runtime::Renderable>>,
    parse_options: ParseRuntimeOptions,
}

thread_local! {
    static ACTIVE_PARTIAL_STATE: RefCell<Option<ActivePartialState>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
struct HostPartialCompiler {
    source: HostPartialSource,
}

impl fmt::Debug for HostPartialCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HostPartialCompiler")
    }
}

impl liquid::partials::PartialCompiler for HostPartialCompiler {
    fn compile(
        self,
        language: Arc<parser::Language>,
    ) -> liquid_core::Result<Box<dyn liquid_core::runtime::PartialStore + Send + Sync>> {
        Ok(Box::new(HostPartialStore {
            language,
            default_file_system: self.source.file_system,
            fallback_cache: Mutex::new(HashMap::new()),
        }))
    }

    fn source(&self) -> &dyn liquid::partials::PartialSource {
        &self.source
    }
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

impl liquid::partials::PartialSource for HostPartialSource {
    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn get<'a>(&'a self, name: &str) -> liquid_core::Result<Option<borrow::Cow<'a, str>>> {
        load_host_partial(self.file_system, name).map(|content| content.map(borrow::Cow::Owned))
    }
}

struct HostPartialStore {
    language: Arc<parser::Language>,
    default_file_system: Opaque<Value>,
    fallback_cache: Mutex<HashMap<String, Arc<dyn liquid_core::runtime::Renderable>>>,
}

impl fmt::Debug for HostPartialStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HostPartialStore")
    }
}

impl liquid_core::runtime::PartialStore for HostPartialStore {
    fn names(&self) -> Vec<&str> {
        Vec::new()
    }

    fn get(
        &self,
        name: &str,
    ) -> liquid_core::Result<Option<Arc<dyn liquid_core::runtime::Renderable>>> {
        if let Some(template) = ACTIVE_PARTIAL_STATE.with(|state| {
            state
                .borrow()
                .as_ref()
                .and_then(|state| state.cache.get(name).cloned())
        }) {
            return Ok(Some(template));
        }

        if let Some(template) = self
            .fallback_cache
            .lock()
            .expect("partial cache mutex should not be poisoned")
            .get(name)
            .cloned()
        {
            return Ok(Some(template));
        }

        let file_system = ACTIVE_PARTIAL_STATE.with(|state| {
            state
                .borrow()
                .as_ref()
                .map(|state| state.file_system)
                .unwrap_or(self.default_file_system)
        });
        let source = ACTIVE_PARTIAL_STATE.with(|state| {
            state
                .borrow()
                .as_ref()
                .and_then(|state| state.source_cache.get(name).cloned())
        });
        let source = match source {
            Some(source) => source,
            None => match load_host_partial(file_system, name)? {
                Some(source) => source,
                None => return Ok(None),
            },
        };
        let parse_options = ACTIVE_PARTIAL_STATE.with(|state| {
            state
                .borrow()
                .as_ref()
                .map(|state| state.parse_options.clone())
                .unwrap_or_else(|| ParseRuntimeOptions::new(false, None))
        });
        let template = parse_source_with_options(&source, &self.language, &parse_options)
            .map(Arc::new)?;
        let template: Arc<dyn liquid_core::runtime::Renderable> = template;

        let inserted = ACTIVE_PARTIAL_STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(state) = state.as_mut() {
                state
                    .source_cache
                    .entry(name.to_owned())
                    .or_insert_with(|| source.clone());
                state.cache.insert(name.to_owned(), template.clone());
                true
            } else {
                false
            }
        });
        if !inserted {
            self.fallback_cache
                .lock()
                .expect("partial cache mutex should not be poisoned")
                .insert(name.to_owned(), template.clone());
        }

        Ok(Some(template))
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

        // `magnus::Value::classname` is unsafe because it borrows a Ruby-backed
        // C string. We call it inside `unsafe` and immediately copy the result
        // into an owned `String`, so we do not keep a borrowed reference that
        // Ruby could later invalidate.
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

#[derive(Clone)]
struct HostFilterEvaluationError {
    class_name: String,
    message: String,
    original: Option<Opaque<Value>>,
}

impl fmt::Debug for HostFilterEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostFilterEvaluationError")
            .field("class_name", &self.class_name)
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for HostFilterEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.class_name, self.message)
    }
}

impl StdError for HostFilterEvaluationError {}

impl HostFilterEvaluationError {
    fn from_host_error(error: &MagnusError) -> Self {
        let message = error.to_string();
        let Some(value) = error.value() else {
            return Self {
                class_name: "RuntimeError".to_owned(),
                message,
                original: None,
            };
        };

        // `magnus::Value::classname` is unsafe because it borrows a Ruby-backed
        // C string. We call it inside `unsafe` and immediately copy the result
        // into an owned `String`, so we do not keep a borrowed reference that
        // Ruby could later invalidate.
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
    let parse_options = ParseRuntimeOptions::new(line_numbers, error_mode);
    let template = parse_template_with_environment(ruby, &source, environment, &parse_options)?;
    let handle = ruby.hash_new();
    handle.aset("source", source)?;
    handle.aset("line_numbers", parse_options.line_numbers)?;
    handle.aset("error_mode", parse_options.error_mode.clone())?;
    handle.aset(
        "environment",
        environment.map_or_else(|| ruby.qnil().as_value(), ReprValue::as_value),
    )?;
    handle.aset("errors", ruby.ary_new())?;
    handle.aset("warnings", ruby.ary_new())?;
    handle.aset("root", build_root_handle(ruby, &template)?)?;
    handle.aset("template", ruby.obj_wrap(template).into_value_with(ruby))?;
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
    ruby: &magnus::Ruby,
    handle: RHash,
) -> Result<Value, MagnusError> {
    let errors: RArray = handle.lookup("errors")?;
    let snapshot = ruby.ary_new();
    for idx in 0..errors.len() {
        let value: Value = errors.entry(idx as isize)?;
        let duplicated = value.funcall::<_, _, Value>("dup", ()).unwrap_or(value);
        snapshot.push(duplicated)?;
    }
    Ok(snapshot.as_value())
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
    let context_handle = RHash::from_value(context_or_assigns);
    let existing_session = context_handle.and_then(context::active_render_session);
    let (live_session, installed_render_session) = if let Some(session) = existing_session {
        (session, false)
    } else {
        values::clear_live_scope_ruby_values();
        let session = liquid_core::runtime::LiveScopeSession::new();
        if let Some(context_handle) = context_handle {
            context::set_render_session_ref(ruby, context_handle, session.clone())?;
        }
        (session, true)
    };

    let errors: RArray = handle.lookup("errors")?;
    errors.clear()?;
    let warnings: RArray = handle.lookup("warnings")?;
    warnings.clear()?;

    values::push_render_error_scope();
    let render_file_system = render_file_system(context_handle);
    let partial_source_cache = literal_partial_source_cache(context_or_assigns);
    let parse_options = ParseRuntimeOptions {
        line_numbers: handle.lookup("line_numbers")?,
        error_mode: handle.lookup("error_mode")?,
    };
    let result = with_active_partial_state(render_file_system, partial_source_cache, parse_options, || {
        let filter_host = build_filter_host(context_or_assigns)?;
        let template = lookup_template(handle)?;
        let lookup_mode = if strict_variables_enabled(context_or_assigns) && !strict {
            values::LookupMode::TrackMissing
        } else {
            values::LookupMode::Materialized
        };
        let globals = Rc::new(globals_from_context(context_or_assigns, lookup_mode)?);
        let strict_variables = strict_variables_enabled(context_or_assigns);
        let strict_filters = strict_filters_enabled(context_or_assigns);
        let lenient_globals = LenientObject::new(globals.as_ref() as &dyn ObjectView);
        let render_globals: &dyn ObjectView = if strict_variables && strict {
            globals.as_ref()
        } else {
            &lenient_globals
        };

        let base_runtime = liquid_core::runtime::RuntimeBuilder::new().set_globals(render_globals);
        let base_runtime = if let Some(partials) = template.partials.as_deref() {
            base_runtime.set_partials(partials)
        } else {
            base_runtime
        };
        let base_runtime = base_runtime.build();
        let runtime = DynamicFilterRuntime::new(&base_runtime, context_or_assigns);
        let callbacks = Rc::new(RubyCallbacks::new(
            context_or_assigns,
            Rc::clone(&globals),
            strict,
        ));
        let fallback_filters =
            filter_host.map(|host| Rc::new(host) as conformance::FallbackFilterRegistry);
        let mut rendered = Vec::new();
        let render_result = conformance::render_to(
            &template.template,
            &mut rendered,
            &runtime,
            &conformance::RenderConfig {
                strict_variables,
                strict_filters,
                callbacks: callbacks.clone(),
                fallback_filters,
                live_scope_session: Some(live_session.clone()),
            },
        );
        let rendered = String::from_utf8(rendered).expect("render should stay valid utf-8");
        for handled_error in callbacks.handled_errors(ruby) {
            errors.push(handled_error)?;
        }

        if let Some(raised_exception) = callbacks.take_raised_exception() {
            return Err(raised_exception);
        }

        match render_result {
            Ok(()) => {
                let mut tracked_errors = values::take_render_error_scope();
                tracked_errors.extend(globals.take_errors());
                let tracked_errors = dedupe_tracked_errors(tracked_errors);
                if let Some(error_value) = tracked_errors.first() {
                    for error in &tracked_errors {
                        errors.push(error.as_ruby_value(ruby))?;
                    }
                    if strict {
                        return Err(error_value.to_magnus_error(ruby));
                    }
                }

                Ok(rendered)
            }
            Err(error) => {
                if let Some(load_error) = host_partial_load_error(&error) {
                    return Err(rebuild_host_error(
                        ruby,
                        &load_error.class_name,
                        &load_error.message,
                        load_error.original,
                    ));
                }
                if let Some(filter_error) = host_filter_evaluation_error(&error) {
                    return Err(rebuild_host_error(
                        ruby,
                        &filter_error.class_name,
                        &filter_error.message,
                        filter_error.original,
                    ));
                }

                let message = error.to_string();
                let mut tracked_errors = values::take_render_error_scope();
                tracked_errors.extend(globals.take_errors());
                let tracked_errors = dedupe_tracked_errors(tracked_errors);
                if strict
                    && (message.contains("Unknown variable") || message.contains("Unknown index"))
                    && !tracked_errors.is_empty()
                {
                    for tracked_error in &tracked_errors {
                        errors.push(tracked_error.as_ruby_value(ruby))?;
                    }
                    return Err(tracked_errors[0].to_magnus_error(ruby));
                }

                errors.push(message.clone())?;
                for tracked_error in tracked_errors {
                    errors.push(tracked_error.as_ruby_value(ruby))?;
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
    });
    values::pop_render_error_scope();

    if installed_render_session {
        live_session.deactivate();
        if let Some(context_handle) = context_handle {
            context::clear_render_session_ref(context_handle)?;
        }
        values::clear_live_scope_ruby_values();
    }

    result
}

fn dedupe_tracked_errors(
    tracked_errors: Vec<values::TrackedRenderError>,
) -> Vec<values::TrackedRenderError> {
    let mut unique = Vec::new();
    for error in tracked_errors {
        if !unique.contains(&error) {
            unique.push(error);
        }
    }
    unique
}

fn parse_template_with_environment(
    ruby: &magnus::Ruby,
    source: &str,
    environment: Option<RHash>,
    parse_options: &ParseRuntimeOptions,
) -> Result<NativeTemplate, MagnusError> {
    let builder = conformance_parser_builder();
    let parser =
        if let Some(file_system) = environment.and_then(|handle| handle.get("file_system")) {
            builder
                .partials(HostPartialCompiler {
                    source: HostPartialSource {
                        file_system: Opaque::from(file_system),
                    },
                })
                .build()
        } else {
            builder.build()
        }
        .map_err(|error| errors::runtime_error(ruby, error.to_string()))?;

    let template = parse_source_with_options(source, &parser.conformance_language(), parse_options)
        .map_err(|error| errors::syntax_error(ruby, error.to_string()))?;

    Ok(NativeTemplate {
        template,
        partials: parser.conformance_partials(),
    })
}

fn conformance_parser_builder() -> ParserBuilder {
    ParserBuilder::new()
        .tag(stdlib::AssignTag)
        .tag(stdlib::BreakTag)
        .tag(stdlib::ContinueTag)
        .tag(stdlib::CycleTag)
        .tag(stdlib::EchoTag)
        .tag(stdlib::IncludeTag)
        .tag(stdlib::IncrementTag)
        .tag(stdlib::DecrementTag)
        .tag(stdlib::RenderTag)
        .block(stdlib::RawBlock)
        .block(stdlib::IfBlock)
        .block(stdlib::UnlessBlock)
        .block(stdlib::IfChangedBlock)
        .block(stdlib::ForBlock)
        .block(stdlib::TableRowBlock)
        .block(stdlib::CommentBlock)
        .block(stdlib::CaptureBlock)
        .block(stdlib::CaseBlock)
}

fn load_host_partial(
    file_system: Opaque<Value>,
    name: &str,
) -> liquid_core::Result<Option<String>> {
    let vm = magnus::Ruby::get().expect("VM should be available");
    let file_system = file_system.get_inner_with(&vm);

    match file_system.funcall::<_, _, Value>("read_template_file", (name,)) {
        Ok(value) => Ok(Some(
            String::try_convert(value)
                .ok()
                .unwrap_or_else(|| value.to_string()),
        )),
        Err(error) => {
            let host_error = HostPartialLoadError::from_host_error(&error);
            if is_missing_host_partial(&host_error) {
                Ok(None)
            } else {
                Err(liquid_core::Error::with_msg("Host partial load failed")
                    .context("requested partial", name.to_owned())
                    .cause(host_error))
            }
        }
    }
}

fn is_missing_host_partial(error: &HostPartialLoadError) -> bool {
    error.class_name == "Liquid::FileSystemError" && error.message.starts_with("No such template ")
}

fn render_file_system(context_handle: Option<RHash>) -> Opaque<Value> {
    let ruby = magnus::Ruby::get().expect("VM should be available");
    context_handle
        .and_then(|handle| handle.get("registers"))
        .and_then(RHash::from_value)
        .and_then(|registers| {
            registers
                .get("file_system")
                .or_else(|| registers.get(ruby.to_symbol("file_system")))
        })
        .filter(|value| !value.is_nil())
        .map(Opaque::from)
        .unwrap_or_else(|| Opaque::from(ruby.qnil().as_value()))
}

fn literal_partial_source_cache(context_or_assigns: Value) -> HashMap<String, String> {
    let Some(context) = current_context(context_or_assigns) else {
        return HashMap::new();
    };

    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let registers: Value = match context.funcall("registers", ()) {
        Ok(registers) => registers,
        Err(_) => return HashMap::new(),
    };
    let cache_value: Value =
        match registers.funcall("[]", (ruby.to_symbol("literal_partial_source_cache"),)) {
            Ok(value) => value,
            Err(_) => return HashMap::new(),
        };
    let Some(cache) = RHash::from_value(cache_value) else {
        return HashMap::new();
    };

    let mut result = HashMap::new();
    let _ = cache.foreach(|key: Value, value: Value| {
        let key = String::try_convert(key).or_else(|_| key.funcall::<_, _, String>("to_s", ()))?;
        let value =
            String::try_convert(value).or_else(|_| value.funcall::<_, _, String>("to_s", ()))?;
        result.insert(key, value);
        Ok(magnus::r_hash::ForEach::Continue)
    });
    result
}

fn parse_source_with_options(
    source: &str,
    language: &Arc<parser::Language>,
    parse_options: &ParseRuntimeOptions,
) -> liquid_core::Result<liquid_core::runtime::Template> {
    // Keep root-template and host-partial parsing on the same option-carrying code path.
    let _ = (parse_options.line_numbers, parse_options.error_mode.as_str());
    parser::parse(source, language).map(liquid_core::runtime::Template::new)
}

fn with_active_partial_state<T>(
    file_system: Opaque<Value>,
    source_cache: HashMap<String, String>,
    parse_options: ParseRuntimeOptions,
    f: impl FnOnce() -> Result<T, MagnusError>,
) -> Result<T, MagnusError> {
    let previous = ACTIVE_PARTIAL_STATE.with(|state| {
        state.borrow_mut().replace(ActivePartialState {
            file_system,
            source_cache,
            cache: HashMap::new(),
            parse_options,
        })
    });

    let result = f();

    ACTIVE_PARTIAL_STATE.with(|state| {
        *state.borrow_mut() = previous;
    });

    result
}

fn host_partial_load_error(error: &liquid_core::Error) -> Option<&HostPartialLoadError> {
    error_chain_find::<HostPartialLoadError>(error)
}

fn host_filter_evaluation_error(error: &liquid_core::Error) -> Option<&HostFilterEvaluationError> {
    error_chain_find::<HostFilterEvaluationError>(error)
}

fn error_chain_find<'a, T: StdError + 'static>(error: &'a dyn StdError) -> Option<&'a T> {
    let mut current = error.source();
    while let Some(cause) = current {
        if let Some(found) = cause.downcast_ref::<T>() {
            return Some(found);
        }
        current = cause.source();
    }
    None
}

fn rebuild_host_error(
    vm: &magnus::Ruby,
    class_name: &str,
    message: &str,
    original: Option<Opaque<Value>>,
) -> MagnusError {
    if class_name.starts_with("Liquid::") {
        if let Ok(class) = lookup_exception_class(vm, class_name) {
            if let Ok(exception) = class.new_instance((message.to_owned(),)) {
                if let Some(exception) = Exception::from_value(exception.as_value()) {
                    return MagnusError::from(exception);
                }
            }
        }
    }

    if let Some(original) = original {
        if let Some(exception) = Exception::from_value(original.get_inner_with(vm)) {
            return MagnusError::from(exception);
        }
    }

    match lookup_exception_class(vm, class_name) {
        Ok(class) => MagnusError::new(class, message.to_owned()),
        Err(_) => MagnusError::new(vm.exception_runtime_error(), message.to_owned()),
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
    _template: &NativeTemplate,
) -> Result<Value, MagnusError> {
    // NativeTemplate stores only the executable runtime template. The Ruby-side
    // AST used for `Template#root` is rebuilt from source, so exposing `[]`
    // here would falsely claim the parsed tree is empty.
    Ok(ruby.qnil().as_value())
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

fn lookup_raw_root_value(context_or_assigns: Value, key: &str) -> Option<Value> {
    if let Some(handle) = RHash::from_value(context_or_assigns) {
        if let Some(scopes) = handle.get("scopes").and_then(RArray::from_value) {
            for idx in (0..scopes.len()).rev() {
                let scope: Value = scopes.entry(idx as isize).ok()?;
                if let Some(value) = lookup_hash_key(scope, key) {
                    return Some(value);
                }
            }
        }

        if let Some(environments) = handle.get("environments").and_then(RArray::from_value) {
            for idx in 0..environments.len() {
                let scope: Value = environments.entry(idx as isize).ok()?;
                if let Some(value) = lookup_hash_key(scope, key) {
                    return Some(value);
                }
            }
        }

        if let Some(static_environments) = handle
            .get("static_environments")
            .and_then(RArray::from_value)
        {
            for idx in 0..static_environments.len() {
                let scope: Value = static_environments.entry(idx as isize).ok()?;
                if let Some(value) = lookup_hash_key(scope, key) {
                    return Some(value);
                }
            }
        }

        return lookup_hash_key(handle.as_value(), key);
    }

    lookup_hash_key(context_or_assigns, key)
}

fn counter_environment_hash(context_or_assigns: Value) -> Option<RHash> {
    if let Some(handle) = RHash::from_value(context_or_assigns) {
        if let Some(environments) = handle.get("environments").and_then(RArray::from_value) {
            return environments.entry::<RHash>(0).ok();
        }

        if let Some(context) = current_context(context_or_assigns) {
            let environments: Value = context.funcall("environments", ()).ok()?;
            let environments = RArray::from_value(environments)?;
            return environments.entry::<RHash>(0).ok();
        }
    }

    None
}

fn counter_assign_hash(context_or_assigns: Value) -> Option<RHash> {
    RHash::from_value(context_or_assigns)
        .and_then(|handle| handle.get("counter_assigns"))
        .and_then(RHash::from_value)
}

fn ensure_counter_assign_hash(context_or_assigns: Value) -> Option<RHash> {
    let handle = RHash::from_value(context_or_assigns)?;
    if let Some(counter_assigns) = handle.get("counter_assigns").and_then(RHash::from_value) {
        return Some(counter_assigns);
    }

    let ruby = magnus::Ruby::get().ok()?;
    let counter_assigns = ruby.hash_new();
    handle.aset("counter_assigns", counter_assigns).ok()?;
    Some(counter_assigns)
}

fn local_scope_contains_key(context_or_assigns: Value, key: &str) -> bool {
    let Some(handle) = RHash::from_value(context_or_assigns) else {
        return false;
    };
    let Some(local_scopes) = handle.get("local_scopes").and_then(RArray::from_value) else {
        return false;
    };

    for idx in (0..local_scopes.len()).rev() {
        let scope: Value = match local_scopes.entry(idx as isize) {
            Ok(scope) => scope,
            Err(_) => continue,
        };
        if lookup_hash_key(scope, key).is_some() {
            return true;
        }
    }

    false
}

fn try_get_counter_assign(
    context_or_assigns: Value,
    path: &[liquid::model::PathElement<'_>],
) -> Option<liquid::model::Value> {
    let first = path.first()?;
    let key = first.value().to_kstr();
    if local_scope_contains_key(context_or_assigns, key.as_str()) {
        return None;
    }

    let counter_assigns = counter_assign_hash(context_or_assigns)?;
    let value = counter_assigns.get(key.as_str())?;
    let owned = values::ruby_to_liquid_value(value).ok()?;

    if path.len() == 1 {
        return Some(owned);
    }

    liquid_core::model::find(&owned, &path[1..])
        .ok()
        .map(|value| value.into_owned())
}

fn lookup_hash_key(scope: Value, key: &str) -> Option<Value> {
    let hash = RHash::from_value(scope)?;
    hash.get(key)
        .or_else(|| key.parse::<i64>().ok().and_then(|index| hash.get(index)))
}

fn strict_variables_enabled(context_or_assigns: Value) -> bool {
    RHash::from_value(context_or_assigns)
        .and_then(|handle| handle.get("strict_variables"))
        .and_then(|value| bool::try_convert(value).ok())
        .unwrap_or(false)
}

fn strict_filters_enabled(context_or_assigns: Value) -> bool {
    RHash::from_value(context_or_assigns)
        .and_then(|handle| handle.get("strict_filters"))
        .and_then(|value| bool::try_convert(value).ok())
        .unwrap_or(false)
}

fn build_filter_host(context_or_assigns: Value) -> Result<Option<HostFilterResolver>, MagnusError> {
    let Some(context) = current_context(context_or_assigns) else {
        return Ok(None);
    };

    let host: Value = context.funcall("strainer", ())?;
    let host_class: RClass = host.funcall("class", ())?;
    let names: RArray = host_class.funcall("filter_method_names", ())?;
    let mut allowed = HashSet::new();
    for idx in 0..names.len() {
        let name_value: Value = names.entry(idx as isize)?;
        let name = String::try_convert(name_value)
            .or_else(|_| name_value.funcall::<_, _, String>("to_s", ()));
        if let Ok(name) = name {
            allowed.insert(name);
        }
    }

    if allowed.is_empty() {
        return Ok(None);
    }

    Ok(Some(HostFilterResolver {
        host,
        allowed,
        context: Some(context),
    }))
}

struct HostFilterResolver {
    host: Value,
    allowed: HashSet<String>,
    context: Option<Value>,
}

impl FallbackFilterResolver for HostFilterResolver {
    fn has_filter(&self, name: &str) -> bool {
        self.allowed.contains(name)
    }

    fn evaluate(
        &self,
        filter: &FilterCall,
        input: &dyn ValueView,
        runtime: &dyn Runtime,
    ) -> liquid_core::Result<LiquidValue> {
        let vm = magnus::Ruby::get().expect("VM should be available");
        let mut args = Vec::new();
        let input_value = values::liquid_to_ruby_value(&vm, &input.to_live_scope_value())
            .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?;
        let _ = values::assign_context_if_supported(input_value, self.context);
        args.push(input_value);

        let filter_args = filter.args();
        for expression in filter_args.positional {
            let value = expression.evaluate(runtime)?;
            let arg_value = values::liquid_to_ruby_value(&vm, &value.to_live_scope_value())
                .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?;
            let _ = values::assign_context_if_supported(arg_value, self.context);
            args.push(arg_value);
        }

        let mut keyword_values: Vec<(_, LiquidValue)> = Vec::new();
        for (name, expression) in filter_args.keyword {
            keyword_values.push((name, expression.evaluate(runtime)?.to_live_scope_value()));
        }

        if !keyword_values.is_empty() {
            let keyword_hash = vm.hash_new();
            for (name, value) in keyword_values {
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

        let result: Value = self
            .host
            .funcall("invoke", &send_args[..])
            .map_err(|error| {
                liquid_core::Error::with_msg(error.to_string())
                    .cause(HostFilterEvaluationError::from_host_error(&error))
            })?;

        let _ = values::assign_context_if_supported(result, self.context);

        values::ruby_filter_result_to_liquid_value(result)
            .map_err(|error| liquid_core::Error::with_msg(error.to_string()))
    }
}

struct DynamicFilterRuntime<'a> {
    inner: &'a dyn Runtime,
    persistent_assigns: Option<RHash>,
    context_or_assigns: Value,
    raw_assigns: RefCell<HashMap<String, Opaque<Value>>>,
}

impl<'a> DynamicFilterRuntime<'a> {
    fn new(inner: &'a dyn Runtime, context_or_assigns: Value) -> Self {
        Self {
            inner,
            persistent_assigns: persistent_assigns_from_context(context_or_assigns),
            context_or_assigns,
            raw_assigns: RefCell::new(HashMap::new()),
        }
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
        let mut roots = self.inner.roots();
        roots.extend(
            self.raw_assigns
                .borrow()
                .keys()
                .cloned()
                .map(KStringCow::from_string),
        );
        roots
    }

    fn try_get(
        &self,
        path: &[liquid::model::PathElement<'_>],
    ) -> Option<liquid::model::ValueCow<'_>> {
        if let Some(value) = self.try_get_raw_assign(path) {
            return Some(value);
        }
        if let Some(value) = try_get_counter_assign(self.context_or_assigns, path) {
            return Some(liquid::model::ValueCow::Owned(value));
        }
        self.inner.try_get(path)
    }

    fn get(
        &self,
        path: &[liquid::model::PathElement<'_>],
    ) -> liquid_core::Result<liquid::model::ValueCow<'_>> {
        if let Some(value) = self.try_get_raw_assign(path) {
            return Ok(value);
        }
        if let Some(value) = try_get_counter_assign(self.context_or_assigns, path) {
            return Ok(liquid::model::ValueCow::Owned(value));
        }
        self.inner.get(path)
    }

    fn set_global(
        &self,
        name: liquid::model::KString,
        val: liquid::model::Value,
    ) -> Option<liquid::model::Value> {
        self.raw_assigns.borrow_mut().remove(name.as_str());
        if let Some(target) = self.persistent_assigns {
            if let Ok(vm) = magnus::Ruby::get() {
                if let Ok(value) = values::liquid_to_ruby_value(&vm, &val) {
                    let _ = target.aset(name.as_str(), value);
                }
            }
        }

        self.inner.set_global(name, val)
    }

    fn set_global_range(
        &self,
        name: liquid::model::KString,
        start: i64,
        stop: i64,
    ) -> Option<liquid::model::Value> {
        self.raw_assigns.borrow_mut().remove(name.as_str());
        if let Some(target) = self.persistent_assigns {
            if let Ok(vm) = magnus::Ruby::get() {
                if let Ok(value) = ruby_range_value(&vm, start, stop) {
                    let _ = target.aset(name.as_str(), value);
                }
            }
        }

        self.inner.set_global_range(name, start, stop)
    }

    fn set_global_alias(
        &self,
        name: liquid::model::KString,
        source: &[liquid::model::PathElement<'_>],
    ) -> bool {
        if source.len() != 1 {
            return false;
        }

        let root_name = source[0].value().to_kstr().into_owned();
        let Some(raw_value) = lookup_raw_root_value(self.context_or_assigns, &root_name) else {
            return false;
        };

        if let Some((start, stop)) = ruby_range_bounds(raw_value) {
            self.raw_assigns.borrow_mut().remove(name.as_str());
            if let Some(target) = self.persistent_assigns {
                let _ = target.aset(name.as_str(), raw_value);
            }
            self.inner.set_global_range(name, start, stop);
            return true;
        }

        if let Some(target) = self.persistent_assigns {
            let _ = target.aset(name.as_str(), raw_value);
        }

        self.raw_assigns
            .borrow_mut()
            .insert(name.into_string(), Opaque::from(raw_value));
        true
    }

    fn set_index(
        &self,
        name: liquid::model::KString,
        val: liquid::model::Value,
    ) -> Option<liquid::model::Value> {
        if let Some(counter_assigns) = ensure_counter_assign_hash(self.context_or_assigns) {
            if let Ok(vm) = magnus::Ruby::get() {
                if let Ok(value) = values::liquid_to_ruby_value(&vm, &val) {
                    let _ = counter_assigns.aset(name.as_str(), value);
                    if let Some(counter_environment) =
                        counter_environment_hash(self.context_or_assigns)
                    {
                        let _ = counter_environment.aset(name.as_str(), value);
                    }
                }
            }
            return None;
        }

        self.inner.set_index(name, val)
    }

    fn get_index<'a>(&'a self, name: &str) -> Option<liquid::model::ValueCow<'a>> {
        if let Some(counter_assigns) = counter_assign_hash(self.context_or_assigns) {
            if let Some(value) = counter_assigns.get(name) {
                if let Ok(value) = values::ruby_to_liquid_value(value) {
                    return Some(liquid::model::ValueCow::Owned(value));
                }
            }
        }

        self.inner.get_index(name)
    }

    fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
        let raw = self.raw_assigns.borrow().get(name).cloned();
        if let Some(raw) = raw {
            let ruby = magnus::Ruby::get().ok()?;
            let raw = raw.get_inner_with(&ruby);
            return ruby_range_bounds(raw);
        }

        self.inner.get_global_range_bounds(name)
    }

    fn registers(&self) -> &liquid_core::runtime::Registers {
        self.inner.registers()
    }
}

impl DynamicFilterRuntime<'_> {
    fn try_get_raw_assign(
        &self,
        path: &[liquid::model::PathElement<'_>],
    ) -> Option<liquid::model::ValueCow<'_>> {
        let first = path.first()?;
        let key = first.value().to_kstr();
        let raw = self.raw_assigns.borrow().get(key.as_str()).cloned()?;
        let ruby = magnus::Ruby::get().ok()?;
        let raw = raw.get_inner_with(&ruby);
        let owned = values::ruby_to_liquid_value(raw).ok()?;

        if path.len() == 1 {
            return Some(liquid::model::ValueCow::Owned(owned));
        }

        liquid_core::model::find(&owned, &path[1..])
            .ok()
            .map(|value| value.into_owned().into())
    }
}

fn ruby_range_value(ruby: &magnus::Ruby, start: i64, stop: i64) -> Result<Value, MagnusError> {
    let range: RClass = ruby.class_object().const_get("Range")?;
    range.funcall("new", (start, stop))
}

fn ruby_range_bounds(value: Value) -> Option<(i64, i64)> {
    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let range: RClass = ruby.class_object().const_get("Range").ok()?;
    if !value.is_kind_of(range) {
        return None;
    }

    let exclude_end: bool = value.funcall("exclude_end?", ()).ok()?;
    if exclude_end {
        return None;
    }

    let start = value
        .funcall::<_, _, Value>("begin", ())
        .ok()
        .and_then(|value| i64::try_convert(value).ok())?;
    let stop = value
        .funcall::<_, _, Value>("end", ())
        .ok()
        .and_then(|value| i64::try_convert(value).ok())?;
    Some((start, stop))
}

struct RubyCallbacks {
    strict: bool,
    tracked_globals: Rc<values::RenderRootObject>,
    recovery: RenderRecoveryState,
    resource_limits: Option<Value>,
}

impl RubyCallbacks {
    fn new(
        context_or_assigns: Value,
        tracked_globals: Rc<values::RenderRootObject>,
        strict: bool,
    ) -> Self {
        Self {
            strict,
            tracked_globals,
            recovery: RenderRecoveryState::new(context_or_assigns),
            resource_limits: resource_limits_from_context(context_or_assigns),
        }
    }

    fn handled_errors(&self, ruby: &magnus::Ruby) -> Vec<Value> {
        self.recovery
            .handled_errors
            .borrow()
            .iter()
            .map(|error| error.to_value(ruby))
            .collect()
    }

    fn take_raised_exception(&self) -> Option<MagnusError> {
        self.recovery.raised_exception.borrow_mut().take()
    }
}

impl ConformanceCallbacks for RubyCallbacks {
    fn handle_render_error(
        &self,
        runtime: &dyn Runtime,
        error: liquid_core::Error,
    ) -> liquid_core::Result<Option<String>> {
        if let Some(load_error) = host_partial_load_error(&error) {
            if self.strict || self.recovery.raised_exception.borrow().is_some() {
                return Err(error);
            }

            let vm = magnus::Ruby::get().expect("VM should be available");
            let message = render_default_error_output(&load_error.message);
            let error_value = build_host_exception_value(&vm, load_error).ok();
            if let Some(error_value) = error_value {
                self.recovery
                    .handled_errors
                    .borrow_mut()
                    .push(HandledRenderError::Value(Opaque::from(error_value)));

                if let Some(renderer) = self.recovery.exception_renderer {
                    return match call_exception_renderer_with_value(renderer, error_value) {
                        Ok(replacement) => Ok(Some(replacement)),
                        Err(error) => {
                            self.recovery
                                .raised_exception
                                .borrow_mut()
                                .replace(wrap_exception_renderer_error(error));
                            Err(liquid_core::Error::with_msg("exception renderer raised"))
                        }
                    };
                }
            } else {
                self.recovery
                    .handled_errors
                    .borrow_mut()
                    .push(HandledRenderError::Message(message.clone()));

                if let Some(renderer) = self.recovery.exception_renderer {
                    return match call_exception_renderer(renderer, &message) {
                        Ok(replacement) => Ok(Some(replacement)),
                        Err(error) => {
                            self.recovery
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

        if self.strict || is_memory_limit_error(&error.to_string()) {
            return Err(error);
        }

        if self.recovery.raised_exception.borrow().is_some() {
            return Err(error);
        }

        if error.to_string().contains("Unknown variable")
            || error.to_string().contains("Unknown index")
        {
            let tracked_errors = self.tracked_globals.take_errors();
            if let Some(first_error) = tracked_errors.first() {
                let vm = magnus::Ruby::get().expect("VM should be available");
                let first_error_value = first_error.as_ruby_value(&vm);
                let first_error_message = first_error.message(&vm);
                self.recovery
                    .handled_errors
                    .borrow_mut()
                    .push(HandledRenderError::Value(Opaque::from(first_error_value)));

                if let Some(renderer) = self.recovery.exception_renderer {
                    return match call_exception_renderer_with_value(renderer, first_error_value) {
                        Ok(replacement) => Ok(Some(replacement)),
                        Err(error) => {
                            self.recovery
                                .raised_exception
                                .borrow_mut()
                                .replace(wrap_exception_renderer_error(error));
                            Err(liquid_core::Error::with_msg("exception renderer raised"))
                        }
                    };
                }

                return Ok(Some(non_strict_error_output(&first_error_message)));
            }
        }

        let message = error.to_string();
        let vm = magnus::Ruby::get().expect("VM should be available");
        if let Ok(error_value) = build_runtime_error_value(&vm, runtime, &message) {
            self.recovery
                .handled_errors
                .borrow_mut()
                .push(HandledRenderError::Value(Opaque::from(error_value)));

            if preserve_partial_output(&message) {
                return Ok(None);
            }

            if let Some(renderer) = self.recovery.exception_renderer {
                return match call_exception_renderer_with_value(renderer, error_value) {
                    Ok(replacement) => Ok(Some(replacement)),
                    Err(error) => {
                        self.recovery
                            .raised_exception
                            .borrow_mut()
                            .replace(wrap_exception_renderer_error(error));
                        Err(liquid_core::Error::with_msg("exception renderer raised"))
                    }
                };
            }

            let rendered: String = error_value
                .funcall("to_s", ())
                .map_err(|error| liquid_core::Error::with_msg(error.to_string()))?;
            return Ok(Some(rendered));
        }

        self.recovery
            .handled_errors
            .borrow_mut()
            .push(HandledRenderError::Message(message.clone()));

        if preserve_partial_output(&message) {
            return Ok(None);
        }

        if let Some(renderer) = self.recovery.exception_renderer {
            return match call_exception_renderer(renderer, &message) {
                Ok(replacement) => Ok(Some(replacement)),
                Err(error) => {
                    self.recovery
                        .raised_exception
                        .borrow_mut()
                        .replace(wrap_exception_renderer_error(error));
                    Err(liquid_core::Error::with_msg("exception renderer raised"))
                }
            };
        }

        Ok(Some(render_default_error_output(&message)))
    }

    fn increment_render_ops(&self, amount: usize) -> liquid_core::Result<()> {
        call_resource_limits_method(self.resource_limits, "increment_render_score", amount)
    }

    fn increment_assign_bytes(&self, amount: usize) -> liquid_core::Result<()> {
        call_resource_limits_method(self.resource_limits, "increment_assign_score", amount)
    }

    fn check_resource_limits(
        &self,
        _runtime: &dyn Runtime,
        rendered_bytes: usize,
    ) -> liquid_core::Result<()> {
        call_resource_limits_method(
            self.resource_limits,
            "increment_write_score",
            rendered_bytes,
        )
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
            Self::Value(value) => {
                let value = value.get_inner_with(ruby);
                value.funcall::<_, _, Value>("dup", ()).unwrap_or(value)
            }
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
    } else if message.starts_with("Liquid error: ") || message.starts_with("Liquid syntax error: ")
    {
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

    let mut message = message.split("\nfrom:").next().unwrap_or(message).trim();
    loop {
        if let Some(stripped) = message.strip_prefix("liquid: ") {
            message = stripped.trim();
            continue;
        }
        if let Some(stripped) = message.strip_prefix("Liquid error: ") {
            message = stripped.trim();
            continue;
        }
        if let Some(stripped) = message.strip_prefix("Liquid syntax error: ") {
            message = stripped.trim();
            continue;
        }
        break;
    }

    message.to_string()
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

fn build_runtime_error_value(
    _vm: &magnus::Ruby,
    runtime: &dyn Runtime,
    message: &str,
) -> Result<Value, MagnusError> {
    let error = wrap_liquid_error(message)?;
    if let Some(template_name) = runtime.name() {
        let template_name = template_name.to_string();
        let _: Value = error.funcall("template_name=", (template_name,))?;
        let line_number = extract_line_number(message).unwrap_or(1) as i64;
        let _: Value = error.funcall("line_number=", (line_number,))?;
    } else if let Some(line_number) = extract_line_number(message) {
        let _: Value = error.funcall("line_number=", (line_number as i64,))?;
    }
    Ok(error)
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

impl fmt::Debug for LenientObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl ValueView for LenientObject<'_> {
    fn as_debug(&self) -> &dyn fmt::Debug {
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

    fn contains_key(&self, index: &str) -> bool {
        self.inner.contains_key(index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        self.inner
            .get(index)
            .map(|value| value as &dyn ValueView)
            .or(Some(&NIL_VALUE as &dyn ValueView))
    }
}
