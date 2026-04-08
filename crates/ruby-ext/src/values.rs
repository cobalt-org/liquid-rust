use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use liquid::model::{ArrayView, DisplayCow, KStringCow, ScalarCow, State, Value as LiquidValue};
use liquid::{Object as LiquidObject, ObjectView, ValueView};
use liquid_core::model::Value as CoreValue;
use magnus::{
    class::Class,
    module::RModule,
    r_array::RArray,
    r_hash::{ForEach, RHash},
    value::BoxValue,
    value::ReprValue,
    Error as MagnusError, Exception, ExceptionClass, IntoValue, Module, RClass, Symbol, TryConvert,
    Value,
};

use crate::callbacks;
use crate::errors;

pub(crate) fn ruby_to_object(value: Value) -> Result<LiquidObject, MagnusError> {
    if let Some(hash) = RHash::from_value(value) {
        let mut object = LiquidObject::new();
        hash.foreach(|key: Value, val: Value| {
            let key = ruby_key_to_string(key)?;
            object.insert(key.into(), ruby_to_liquid_value(val)?);
            Ok(ForEach::Continue)
        })?;
        Ok(object)
    } else {
        Err(errors::argument_error(
            &magnus::Ruby::get().expect("Ruby VM should be available"),
            "expected a Hash-like object",
        ))
    }
}

pub(crate) type RenderObject = HashMap<String, RenderValue>;

// Live scope values are serialized as marker objects so Rust-owned Liquid model
// values can later be rehydrated back into the original Ruby objects.
const LIVE_SCOPE_OPAQUE_MARKER_KEY: &str = "__liquid_ruby_live_scope__";
const LIVE_SCOPE_OPAQUE_TOKEN_KEY: &str = "__liquid_ruby_live_scope_token__";

thread_local! {
    static LIVE_SCOPE_RUBY_VALUES: RefCell<HashMap<String, LiveScopeRubyValue>> = RefCell::new(HashMap::new());
    static RENDER_ERROR_SCOPES: RefCell<Vec<Vec<TrackedRenderError>>> = const { RefCell::new(Vec::new()) };
}

static LIVE_SCOPE_RUBY_VALUE_SEQ: AtomicU64 = AtomicU64::new(1);

struct LiveScopeRubyValue {
    original: BoxValue<Value>,
    semantic: Option<BoxValue<Value>>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum TrackedRenderError {
    Message(String),
    Exception { class_name: String, message: String },
}

pub(crate) fn push_render_error_scope() {
    RENDER_ERROR_SCOPES.with(|scopes| scopes.borrow_mut().push(Vec::new()));
}

pub(crate) fn take_render_error_scope() -> Vec<TrackedRenderError> {
    RENDER_ERROR_SCOPES.with(|scopes| {
        scopes
            .borrow_mut()
            .last_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    })
}

pub(crate) fn pop_render_error_scope() {
    RENDER_ERROR_SCOPES.with(|scopes| {
        scopes.borrow_mut().pop();
    });
}

fn record_render_scope_error(error: &TrackedRenderError) {
    RENDER_ERROR_SCOPES.with(|scopes| {
        if let Some(current) = scopes.borrow_mut().last_mut() {
            current.push(error.clone());
        }
    });
}

impl TrackedRenderError {
    pub(crate) fn from_magnus_error(error: MagnusError) -> Self {
        if let Some(value) = error.value() {
            let class_name = unsafe { value.classname() }.to_string();
            let message = value
                .funcall::<_, _, String>("message", ())
                .unwrap_or_else(|_| error.to_string());
            Self::Exception {
                class_name: class_name.clone(),
                message: normalize_exception_message(&class_name, message),
            }
        } else {
            Self::Message(error.to_string())
        }
    }

    pub(crate) fn as_ruby_value(&self, ruby: &magnus::Ruby) -> Value {
        match self {
            Self::Message(message) => ruby.str_new(message).as_value(),
            Self::Exception {
                class_name,
                message,
            } => build_exception_value(ruby, class_name, message),
        }
    }

    pub(crate) fn message(&self, ruby: &magnus::Ruby) -> String {
        match self {
            Self::Message(message) => message.clone(),
            Self::Exception { message, .. } => {
                if let Ok(value) = String::try_convert(self.as_ruby_value(ruby)) {
                    value
                } else {
                    message.clone()
                }
            }
        }
    }

    pub(crate) fn to_magnus_error(&self, ruby: &magnus::Ruby) -> MagnusError {
        match self {
            Self::Message(message) => {
                MagnusError::new(ruby.exception_runtime_error(), message.clone())
            }
            Self::Exception {
                class_name,
                message,
            } => Exception::from_value(build_exception_value(ruby, class_name, message))
                .map(MagnusError::from)
                .unwrap_or_else(|| {
                    MagnusError::new(ruby.exception_runtime_error(), message.clone())
                }),
        }
    }
}

impl std::fmt::Debug for TrackedRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => f.debug_tuple("Message").field(message).finish(),
            Self::Exception {
                class_name,
                message,
            } => f
                .debug_struct("Exception")
                .field("class_name", class_name)
                .field("message", message)
                .finish(),
        }
    }
}

fn normalize_exception_message(class_name: &str, message: String) -> String {
    if !class_name.starts_with("Liquid::") {
        return message;
    }

    message
        .strip_prefix("Liquid syntax error: ")
        .or_else(|| message.strip_prefix("Liquid error: "))
        .unwrap_or(&message)
        .to_owned()
}

fn build_exception_value(ruby: &magnus::Ruby, class_name: &str, message: &str) -> Value {
    lookup_exception_class(ruby, class_name)
        .and_then(|class| class.new_instance((message.to_owned(),)))
        .map(ReprValue::as_value)
        .unwrap_or_else(|_| {
            ruby.exception_runtime_error()
                .new_instance((message.to_owned(),))
                .unwrap()
                .as_value()
        })
}

fn lookup_exception_class(
    ruby: &magnus::Ruby,
    class_name: &str,
) -> Result<ExceptionClass, MagnusError> {
    let mut current = ruby.class_object().as_value();
    for segment in class_name.split("::").filter(|segment| !segment.is_empty()) {
        current = current.funcall("const_get", (segment,))?;
    }

    ExceptionClass::from_value(current)
        .ok_or_else(|| MagnusError::new(ruby.exception_type_error(), class_name.to_owned()))
}

#[derive(Debug)]
pub(crate) struct RenderRootObject {
    scopes: Vec<RenderScope>,
    errors: RefCell<Vec<TrackedRenderError>>,
    lookup_mode: LookupMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LookupMode {
    Materialized,
    TrackMissing,
}

#[derive(Debug)]
enum RenderScope {
    Materialized(RenderObject),
    Dynamic(RenderDynamicObject),
}

impl RenderRootObject {
    pub(crate) fn from_render_object(object: RenderObject) -> Self {
        Self {
            scopes: vec![RenderScope::Materialized(object)],
            errors: RefCell::new(Vec::new()),
            lookup_mode: LookupMode::Materialized,
        }
    }

    pub(crate) fn from_liquid_object(object: LiquidObject) -> Self {
        Self::from_render_object(liquid_to_render_object(object))
    }

    pub(crate) fn from_value(value: Value) -> Result<Self, MagnusError> {
        Self::from_value_with_mode(value, LookupMode::Materialized)
    }

    pub(crate) fn from_value_with_mode(
        value: Value,
        mode: LookupMode,
    ) -> Result<Self, MagnusError> {
        Self::from_value_with_mode_and_context(value, mode, None)
    }

    pub(crate) fn from_value_with_mode_and_context(
        value: Value,
        mode: LookupMode,
        context: Option<Value>,
    ) -> Result<Self, MagnusError> {
        Ok(Self {
            scopes: vec![render_scope_from_value(value, mode, context)?],
            errors: RefCell::new(Vec::new()),
            lookup_mode: mode,
        })
    }

    pub(crate) fn from_values(values: Vec<Value>) -> Result<Self, MagnusError> {
        Self::from_values_with_mode(values, LookupMode::Materialized)
    }

    pub(crate) fn from_values_with_mode(
        values: Vec<Value>,
        mode: LookupMode,
    ) -> Result<Self, MagnusError> {
        Self::from_values_with_mode_and_context(values, mode, None)
    }

    pub(crate) fn from_values_with_mode_and_context(
        values: Vec<Value>,
        mode: LookupMode,
        context: Option<Value>,
    ) -> Result<Self, MagnusError> {
        let mut scopes = Vec::with_capacity(values.len());
        for value in values {
            scopes.push(render_scope_from_value(value, mode, context)?);
        }
        Ok(Self {
            scopes,
            errors: RefCell::new(Vec::new()),
            lookup_mode: mode,
        })
    }

    fn merged_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for scope in &self.scopes {
            for key in scope.keys_owned() {
                if !keys.iter().any(|existing| existing == &key) {
                    keys.push(key);
                }
            }
        }
        keys
    }

    pub(crate) fn take_errors(&self) -> Vec<TrackedRenderError> {
        let mut errors = std::mem::take(&mut *self.errors.borrow_mut());
        errors.extend(self.scopes.iter().flat_map(RenderScope::take_errors));
        errors
    }

    fn record_missing(&self, index: &str) {
        if self.lookup_mode != LookupMode::TrackMissing {
            return;
        }

        self.errors
            .borrow_mut()
            .push(TrackedRenderError::Message(format!(
                "liquid: Unknown variable\n  with:\n    requested variable={index}"
            )));
    }
}

impl ValueView for RenderRootObject {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(self.to_value().render().to_string()))
    }

    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(self.to_value().source().to_string()))
    }

    fn type_name(&self) -> &'static str {
        "object"
    }

    fn query_state(&self, state: State) -> bool {
        !matches!(state, State::DefaultValue | State::Empty | State::Blank)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        KStringCow::from_string(self.to_value().to_kstr().into_owned().to_string())
    }

    fn to_value(&self) -> LiquidValue {
        let mut object = LiquidObject::new();
        for key in self.merged_keys() {
            if let Some(value) = self.get(&key) {
                object.insert(key.into(), value.to_value());
            }
        }
        LiquidValue::Object(object)
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for RenderRootObject {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.merged_keys().len() as i64
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(
            self.merged_keys()
                .into_iter()
                .map(|key| KStringCow::from_string(key)),
        )
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let keys = self.merged_keys();
        Box::new(keys.into_iter().filter_map(|key| self.get(&key)))
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let keys = self.merged_keys();
        Box::new(keys.into_iter().filter_map(|key| {
            self.get(&key)
                .map(|value| (KStringCow::from_string(key), value))
        }))
    }

    fn contains_key(&self, index: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(index))
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get_for_root(index) {
                return Some(value);
            }
        }

        self.record_missing(index);
        None
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RenderValue {
    Nil,
    Scalar(RenderScalar),
    Host(RenderHostValue),
    InlineError {
        rendered: String,
        error: TrackedRenderError,
    },
    Array(Vec<RenderValue>),
    // Materialized object data.
    //
    // Use this when we already have the full object as native key/value data.
    // Example:
    //   { "product" => { "title" => "Hat" } }
    // becomes:
    //   Object({ "title" => Scalar("Hat") })
    Object(RenderObject),
    // Runtime-backed object data.
    //
    // Use this when the value behaves like an object, but its fields must be
    // resolved by calling back into the original object at lookup time instead
    // of being copied into a native map first.
    //
    // Example:
    //   settings.zero
    // where `settings` is a Drop-like object that answers `"zero"` dynamically.
    //
    // In Shopify Liquid, expressions like:
    //   list[settings.zero]
    // must ask the original object for `"zero"` during lookup. If we flatten
    // that object too early, we lose that behavior and the lookup can resolve
    // to the wrong value or `nil`.
    DynamicObject(RenderDynamicObject),
}

#[derive(Clone, Debug)]
pub(crate) struct RenderScalar {
    semantic: ScalarCow<'static>,
    rendered: Option<String>,
}

impl RenderScalar {
    fn new(semantic: ScalarCow<'static>, rendered: Option<String>) -> Self {
        Self { semantic, rendered }
    }
}

impl ValueView for RenderScalar {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        if let Some(rendered) = &self.rendered {
            DisplayCow::Owned(Box::new(rendered.clone()))
        } else {
            self.semantic.render()
        }
    }

    fn source(&self) -> DisplayCow<'_> {
        if let Some(rendered) = &self.rendered {
            DisplayCow::Owned(Box::new(rendered.clone()))
        } else {
            self.semantic.source()
        }
    }

    fn type_name(&self) -> &'static str {
        self.semantic.type_name()
    }

    fn query_state(&self, state: State) -> bool {
        self.semantic.query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        if let Some(rendered) = &self.rendered {
            KStringCow::from_string(rendered.clone().into())
        } else {
            self.semantic.to_kstr()
        }
    }

    fn to_value(&self) -> LiquidValue {
        self.semantic.to_value()
    }

    fn as_scalar(&self) -> Option<ScalarCow<'_>> {
        Some(self.semantic.as_ref())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RenderHostValue {
    original: Value,
    semantic_source: Option<Value>,
    semantic: Box<RenderValue>,
}

impl RenderHostValue {
    fn new(original: Value, semantic_source: Option<Value>, semantic: RenderValue) -> Self {
        Self {
            original,
            semantic_source,
            semantic: Box::new(semantic),
        }
    }
}

impl ValueView for RenderHostValue {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        self.semantic.render()
    }

    fn source(&self) -> DisplayCow<'_> {
        self.semantic.source()
    }

    fn type_name(&self) -> &'static str {
        self.semantic.type_name()
    }

    fn query_state(&self, state: State) -> bool {
        self.semantic.query_state(state)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        self.semantic.to_kstr()
    }

    fn to_value(&self) -> LiquidValue {
        self.semantic.to_value()
    }

    fn to_live_scope_value(&self) -> LiquidValue {
        live_scope_opaque_snapshot(self.original, self.semantic_source)
    }

    fn as_scalar(&self) -> Option<ScalarCow<'_>> {
        self.semantic.as_scalar()
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        self.semantic.as_array()
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        self.semantic.as_object()
    }
}

#[derive(Debug)]
pub(crate) struct RenderDynamicObject {
    value: Value,
    context: Option<Value>,
    // Stable cache for non-volatile lookups.
    cache: Rc<RefCell<HashMap<String, Box<RenderValue>>>>,
    // Old volatile entries stay alive here so previously returned references
    // remain valid after refreshes.
    retained_cache: Rc<RefCell<Vec<Box<RenderValue>>>>,
    volatile_keys: Rc<RefCell<HashSet<String>>>,
    // Arrays are materialized once and then exposed by shared reference.
    array_cache: Rc<OnceCell<Box<[RenderValue]>>>,
    errors: Rc<RefCell<Vec<TrackedRenderError>>>,
    lookup_mode: LookupMode,
}

impl Clone for RenderDynamicObject {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            context: self.context,
            cache: Rc::clone(&self.cache),
            retained_cache: Rc::clone(&self.retained_cache),
            volatile_keys: Rc::clone(&self.volatile_keys),
            array_cache: Rc::clone(&self.array_cache),
            errors: Rc::clone(&self.errors),
            lookup_mode: self.lookup_mode,
        }
    }
}

impl RenderDynamicObject {
    fn new(value: Value, lookup_mode: LookupMode, context: Option<Value>) -> Self {
        Self {
            value,
            context,
            cache: Rc::new(RefCell::new(HashMap::new())),
            retained_cache: Rc::new(RefCell::new(Vec::new())),
            volatile_keys: Rc::new(RefCell::new(HashSet::new())),
            array_cache: Rc::new(OnceCell::new()),
            errors: Rc::new(RefCell::new(Vec::new())),
            lookup_mode,
        }
    }

    fn render_string(&self) -> String {
        liquid_utils_to_s(self.value).unwrap_or_else(|_| self.value.to_string())
    }

    fn lookup_value(&self, index: &str) -> Result<Option<Value>, MagnusError> {
        self.lookup_value_with_missing(index, true)
    }

    fn lookup_value_without_missing(&self, index: &str) -> Result<Option<Value>, MagnusError> {
        self.lookup_value_with_missing(index, false)
    }

    fn lookup_value_with_missing(
        &self,
        index: &str,
        record_missing: bool,
    ) -> Result<Option<Value>, MagnusError> {
        let mut present_via_key_check = false;
        let integer_index = index.parse::<i64>().ok();
        let mut use_integer_index = false;

        if !hash_has_dynamic_default(self.value)? && self.value.respond_to("key?", false)? {
            let has_key: bool = self.value.funcall("key?", (index,))?;
            if !has_key {
                if let Some(integer_index) = integer_index {
                    let has_integer_key: bool = self.value.funcall("key?", (integer_index,))?;
                    if has_integer_key {
                        present_via_key_check = true;
                        use_integer_index = true;
                    } else {
                        if record_missing {
                            self.record_missing(index);
                        }
                        return Ok(None);
                    }
                } else {
                    if record_missing {
                        self.record_missing(index);
                    }
                    return Ok(None);
                }
            } else {
                present_via_key_check = true;
            }
        }

        if self.value.respond_to("[]", false)? {
            let result: Value = if use_integer_index {
                self.value
                    .funcall("[]", (integer_index.expect("integer key should exist"),))?
            } else {
                self.value.funcall("[]", (index,))?
            };
            if result.is_nil() {
                if present_via_key_check {
                    Ok(Some(result))
                } else {
                    if record_missing {
                        self.record_missing(index);
                    }
                    Ok(None)
                }
            } else {
                resolve_dynamic_value(self.value, self.context, index, result).map(Some)
            }
        } else {
            if record_missing {
                self.record_missing(index);
            }
            Ok(None)
        }
    }

    fn record_error(&self, error: MagnusError) {
        self.errors
            .borrow_mut()
            .push(TrackedRenderError::from_magnus_error(error));
    }

    fn record_missing(&self, index: &str) {
        let is_drop_like = self.value.respond_to("invoke_drop", false).unwrap_or(false);
        if self.lookup_mode != LookupMode::TrackMissing && !is_drop_like {
            return;
        }

        let message = if is_drop_like {
            format!("liquid: Undefined drop method\n  with:\n    requested variable={index}")
        } else {
            format!("liquid: Unknown variable\n  with:\n    requested variable={index}")
        };
        self.errors
            .borrow_mut()
            .push(TrackedRenderError::Message(message));
    }

    fn take_errors(&self) -> Vec<TrackedRenderError> {
        let mut errors = std::mem::take(&mut *self.errors.borrow_mut());
        let cache = self.cache.borrow();
        for value in cache.values() {
            errors.extend(value.take_errors());
        }
        let retained_cache = self.retained_cache.borrow();
        for value in retained_cache.iter() {
            errors.extend(value.take_errors());
        }
        errors
    }

    fn keys_owned(&self) -> Vec<String> {
        if let Some(hash) = RHash::from_value(self.value) {
            let mut keys = Vec::new();
            let _ = hash.foreach(|key: Value, _val: Value| {
                keys.push(ruby_key_to_string(key)?);
                Ok(ForEach::Continue)
            });
            return keys;
        }

        if !self.value.respond_to("invoke_drop", false).unwrap_or(false) {
            return Vec::new();
        }

        let class: Value = match self.value.funcall("class", ()) {
            Ok(class) => class,
            Err(_) => return Vec::new(),
        };
        let methods: Value = match class.funcall("invokable_methods", ()) {
            Ok(methods) => methods,
            Err(_) => return Vec::new(),
        };
        let methods: Value = match methods.funcall("to_a", ()) {
            Ok(methods) => methods,
            Err(_) => return Vec::new(),
        };
        let Some(array) = RArray::from_value(methods) else {
            return Vec::new();
        };

        let mut keys = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let method: Value = match array.entry(idx as isize) {
                Ok(method) => method,
                Err(_) => continue,
            };
            if let Ok(name) = String::try_convert(method) {
                if name != "to_liquid" {
                    keys.push(name);
                }
            }
        }
        keys
    }

    fn ensure_array_cache_loaded(&self) -> Result<bool, MagnusError> {
        if self.array_cache.get().is_some() {
            return Ok(true);
        }

        if !self.value.respond_to("each", false)? {
            return Ok(false);
        }

        let enumerator: Value = self.value.funcall("to_enum", ("each",))?;
        let array_value: Value = enumerator.funcall("to_a", ())?;
        let Some(array) = RArray::from_value(array_value) else {
            return Ok(false);
        };

        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_to_render_value_with_context(
                item,
                self.lookup_mode,
                self.context,
            )?);
        }
        // Array iteration only needs immutable access after the first load.
        let _ = self.array_cache.set(items.into_boxed_slice());
        Ok(true)
    }

    fn lookup_result_requires_refresh(value: Value) -> Result<bool, MagnusError> {
        if ruby_value_is_direct_scalar(value) {
            return Ok(false);
        }

        let mut visited = HashSet::new();
        value_contains_contextual_descendants(value, &mut visited)
    }

    fn should_suppress_lookup_error(error: &MagnusError, drop_like: bool) -> bool {
        if !drop_like {
            return false;
        }

        error
            .value()
            .map(|value| unsafe { value.classname() }.as_ref() == "Liquid::UndefinedDropMethod")
            .unwrap_or(false)
    }

    fn cached_value<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        let cache = self.cache.borrow();
        let value = cache.get(index)?;
        let ptr: *const RenderValue = &**value;
        drop(cache);

        // SAFETY: cached values live in `Box` allocations. When a key is refreshed,
        // we move the previous `Box` into `retained_cache` instead of dropping it,
        // so references previously returned for the lifetime of `self` remain valid.
        Some(unsafe { &*ptr } as &dyn ValueView)
    }

    fn materialize_lookup<'s>(
        &'s self,
        index: &str,
        value: Value,
    ) -> Result<Option<&'s dyn ValueView>, MagnusError> {
        let drop_like = self.value.respond_to("invoke_drop", false)?;
        let requires_refresh = Self::lookup_result_requires_refresh(value)?;
        let refresh_each_lookup = drop_like || requires_refresh;
        if refresh_each_lookup {
            self.volatile_keys.borrow_mut().insert(index.to_string());
        }
        let render_value = ruby_to_render_value_with_context(value, self.lookup_mode, self.context)?;

        let mut cache = self.cache.borrow_mut();
        if refresh_each_lookup {
            if let Some(previous) = cache.insert(index.to_string(), Box::new(render_value)) {
                self.retained_cache.borrow_mut().push(previous);
            }
        } else {
            cache
                .entry(index.to_string())
                .or_insert_with(|| Box::new(render_value));
        }
        drop(cache);
        Ok(self.cached_value(index))
    }

    fn materialize_inline_error<'s>(
        &'s self,
        index: &str,
        error: TrackedRenderError,
        rendered: String,
    ) -> Option<&'s dyn ValueView> {
        self.volatile_keys.borrow_mut().insert(index.to_string());

        let mut cache = self.cache.borrow_mut();
        if let Some(previous) = cache.insert(
            index.to_string(),
            Box::new(RenderValue::InlineError { rendered, error }),
        ) {
            self.retained_cache.borrow_mut().push(previous);
        }
        drop(cache);
        self.cached_value(index)
    }

    fn get_for_root<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        let drop_like = match self.value.respond_to("invoke_drop", false) {
            Ok(drop_like) => drop_like,
            Err(error) => {
                let tracked = TrackedRenderError::from_magnus_error(error);
                let rendered = render_error_placeholder(&tracked);
                record_render_scope_error(&tracked);
                return self.materialize_inline_error(index, tracked, rendered);
            }
        };

        let volatile = self.volatile_keys.borrow().contains(index);
        if !drop_like && !volatile {
            if let Some(value) = self.cached_value(index) {
                return Some(value);
            }
        }

        match self.lookup_value_without_missing(index) {
            Ok(Some(value)) => match self.materialize_lookup(index, value) {
                Ok(value) => value,
                Err(error) => {
                    let suppress = Self::should_suppress_lookup_error(&error, drop_like);
                    let tracked = TrackedRenderError::from_magnus_error(error);
                    let rendered = render_error_placeholder(&tracked);
                    if suppress {
                        self.errors.borrow_mut().push(tracked);
                        Some(&LiquidValue::Nil as &dyn ValueView)
                    } else {
                        record_render_scope_error(&tracked);
                        self.materialize_inline_error(index, tracked, rendered)
                    }
                }
            },
            Ok(None) => None,
            Err(error) => {
                let suppress = Self::should_suppress_lookup_error(&error, drop_like);
                let tracked = TrackedRenderError::from_magnus_error(error);
                let rendered = render_error_placeholder(&tracked);
                if suppress {
                    self.errors.borrow_mut().push(tracked);
                    Some(&LiquidValue::Nil as &dyn ValueView)
                } else {
                    record_render_scope_error(&tracked);
                    self.materialize_inline_error(index, tracked, rendered)
                }
            }
        }
    }
}

fn liquid_utils_to_s(value: Value) -> Result<String, MagnusError> {
    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let liquid: RModule = ruby.class_object().const_get("Liquid")?;
    let utils: RModule = liquid.const_get("Utils")?;
    utils.funcall("to_s", (value,))
}

impl RenderScope {
    fn keys_owned(&self) -> Vec<String> {
        match self {
            Self::Materialized(object) => object.keys().cloned().collect(),
            Self::Dynamic(object) => object.keys_owned(),
        }
    }

    fn contains_key(&self, index: &str) -> bool {
        match self {
            Self::Materialized(object) => object.contains_key(index),
            Self::Dynamic(object) => ObjectView::contains_key(object, index),
        }
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        match self {
            Self::Materialized(object) => object.get(index).map(|value| value as &dyn ValueView),
            Self::Dynamic(object) => ObjectView::get(object, index),
        }
    }

    fn get_for_root<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        match self {
            Self::Materialized(object) => object.get(index).map(|value| value as &dyn ValueView),
            Self::Dynamic(object) => object.get_for_root(index),
        }
    }

    fn take_errors(&self) -> Vec<TrackedRenderError> {
        match self {
            Self::Materialized(object) => take_object_errors(object),
            Self::Dynamic(object) => object.take_errors(),
        }
    }
}

impl ValueView for RenderDynamicObject {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(self.render_string()))
    }

    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Owned(Box::new(self.render_string()))
    }

    fn type_name(&self) -> &'static str {
        "object"
    }

    fn query_state(&self, state: State) -> bool {
        matches!(state, State::Truthy)
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        KStringCow::from_string(self.render_string())
    }

    fn to_value(&self) -> LiquidValue {
        let keys = self.keys_owned();
        let is_hash = RHash::from_value(self.value).is_some();
        if keys.is_empty() && !is_hash {
            LiquidValue::scalar(self.render_string())
        } else {
            let mut object = LiquidObject::new();
            for key in keys {
                if let Some(value) = ObjectView::get(self, &key) {
                    object.insert(key.into(), value.to_value());
                }
            }
            LiquidValue::Object(object)
        }
    }
    fn to_live_scope_value(&self) -> LiquidValue {
        live_scope_opaque_snapshot(self.value, None)
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        if RHash::from_value(self.value).is_some() {
            return None;
        }

        match self.value.respond_to("each", false) {
            Ok(true) => Some(self),
            Ok(false) => None,
            Err(error) => {
                self.record_error(error);
                None
            }
        }
    }
}

impl ObjectView for RenderDynamicObject {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.keys_owned().len() as i64
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(self.keys_owned().into_iter().map(KStringCow::from_string))
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        let keys = self.keys_owned();
        Box::new(
            keys.into_iter()
                .filter_map(|key| ObjectView::get(self, &key)),
        )
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        let keys = self.keys_owned();
        Box::new(keys.into_iter().filter_map(|key| {
            ObjectView::get(self, &key).map(|value| (KStringCow::from_string(key), value))
        }))
    }

    fn contains_key(&self, index: &str) -> bool {
        let drop_like = self.value.respond_to("invoke_drop", false).unwrap_or(false);
        match self.lookup_value(index) {
            Ok(None) if self.lookup_mode == LookupMode::TrackMissing => true,
            Ok(value) => value.is_some(),
            Err(error) => {
                let suppress = Self::should_suppress_lookup_error(&error, drop_like);
                self.record_error(error);
                suppress
            }
        }
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        let drop_like = self.value.respond_to("invoke_drop", false).unwrap_or(false);

        let volatile = self.volatile_keys.borrow().contains(index);
        if !drop_like && !volatile {
            if let Some(value) = self.cached_value(index) {
                return Some(value);
            }
        }

        let value = match self.lookup_value(index) {
            Ok(None) if self.lookup_mode == LookupMode::TrackMissing => {
                return Some(&LiquidValue::Nil as &dyn ValueView);
            }
            Ok(value) => value?,
            Err(error) => {
                let suppress = Self::should_suppress_lookup_error(&error, drop_like);
                let tracked = TrackedRenderError::from_magnus_error(error);
                if suppress {
                    self.errors.borrow_mut().push(tracked);
                    return Some(&LiquidValue::Nil as &dyn ValueView);
                }
                let rendered = render_error_placeholder(&tracked);
                record_render_scope_error(&tracked);
                return self.materialize_inline_error(index, tracked, rendered);
            }
        };
        match self.materialize_lookup(index, value) {
            Ok(value) => value,
            Err(error) => {
                let suppress = Self::should_suppress_lookup_error(&error, drop_like);
                let tracked = TrackedRenderError::from_magnus_error(error);
                if suppress {
                    self.errors.borrow_mut().push(tracked);
                    return Some(&LiquidValue::Nil as &dyn ValueView);
                }
                let rendered = render_error_placeholder(&tracked);
                record_render_scope_error(&tracked);
                self.materialize_inline_error(index, tracked, rendered)
            }
        }
    }
}

impl ArrayView for RenderDynamicObject {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        match self.ensure_array_cache_loaded() {
            Ok(true) => self
                .array_cache
                .get()
                .map(|items| items.len() as i64)
                .unwrap_or(0),
            Ok(false) => 0,
            Err(error) => {
                self.record_error(error);
                0
            }
        }
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        match self.ensure_array_cache_loaded() {
            Ok(true) => {
                let items = self
                    .array_cache
                    .get()
                    .expect("array cache should be populated after loading");
                Box::new(items.iter().map(|item| item as &dyn ValueView))
            }
            Ok(false) => Box::new(std::iter::empty()),
            Err(error) => {
                self.record_error(error);
                Box::new(std::iter::empty())
            }
        }
    }

    fn contains_key(&self, index: i64) -> bool {
        let size = ArrayView::size(self);
        let index = if index < 0 { size + index } else { index };
        index >= 0 && index < size
    }

    fn get(&self, index: i64) -> Option<&dyn ValueView> {
        if let Err(error) = self.ensure_array_cache_loaded() {
            self.record_error(error);
            return None;
        }

        let items = self.array_cache.get()?;
        let size = items.len() as i64;
        let index = if index < 0 { size + index } else { index };
        items
            .get(index as usize)
            .map(|value| value as &dyn ValueView)
    }

    fn slice(&self, from: usize, to: Option<usize>) -> Vec<LiquidValue> {
        if (from != 0 || to.is_some())
            && self.value.respond_to("load_slice", false).unwrap_or(false)
        {
            let loaded = (|| -> Result<Vec<LiquidValue>, MagnusError> {
                let array_value: Value = self
                    .value
                    .funcall("load_slice", (from as i64, to.map(|value| value as i64)))?;
                let Some(array) = RArray::from_value(array_value) else {
                    return Ok(Vec::new());
                };

                let mut items = Vec::with_capacity(array.len());
                for idx in 0..array.len() {
                    let item: Value = array.entry(idx as isize)?;
                    items.push(
                        ruby_to_render_value_with_context(item, self.lookup_mode, self.context)?
                            .to_value(),
                    );
                }
                Ok(items)
            })();

            match loaded {
                Ok(items) => return items,
                Err(error) => self.record_error(error),
            }
        }

        let take = to.map(|end| end.saturating_sub(from));
        let iter = ArrayView::values(self).skip(from);
        match take {
            Some(limit) => iter
                .take(limit)
                .map(|value: &dyn ValueView| value.to_value())
                .collect(),
            None => iter.map(|value: &dyn ValueView| value.to_value()).collect(),
        }
    }
}

impl ValueView for RenderValue {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> DisplayCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.render(),
            Self::Scalar(value) => value.render(),
            Self::Host(value) => value.render(),
            Self::InlineError { rendered, .. } => DisplayCow::Owned(Box::new(rendered.clone())),
            Self::Array(values) => values.render(),
            Self::Object(values) => values.render(),
            Self::DynamicObject(value) => value.render(),
        }
    }

    fn source(&self) -> DisplayCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.source(),
            Self::Scalar(value) => value.source(),
            Self::Host(value) => value.source(),
            Self::InlineError { rendered, .. } => DisplayCow::Owned(Box::new(rendered.clone())),
            Self::Array(values) => values.source(),
            Self::Object(values) => values.source(),
            Self::DynamicObject(value) => value.source(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => LiquidValue::Nil.type_name(),
            Self::Scalar(value) => value.type_name(),
            Self::Host(value) => value.type_name(),
            Self::InlineError { .. } => LiquidValue::Nil.type_name(),
            Self::Array(values) => values.type_name(),
            Self::Object(values) => values.type_name(),
            Self::DynamicObject(value) => value.type_name(),
        }
    }

    fn query_state(&self, state: State) -> bool {
        match self {
            Self::Nil => LiquidValue::Nil.query_state(state),
            Self::Scalar(value) => value.query_state(state),
            Self::Host(value) => value.query_state(state),
            Self::InlineError { .. } => LiquidValue::Nil.query_state(state),
            Self::Array(values) => values.query_state(state),
            Self::Object(values) => values.query_state(state),
            Self::DynamicObject(value) => value.query_state(state),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.to_kstr(),
            Self::Scalar(value) => value.to_kstr(),
            Self::Host(value) => value.to_kstr(),
            Self::InlineError { rendered, .. } => KStringCow::from_string(rendered.clone().into()),
            Self::Array(values) => values.to_kstr(),
            Self::Object(values) => values.to_kstr(),
            Self::DynamicObject(value) => value.to_kstr(),
        }
    }

    fn to_value(&self) -> LiquidValue {
        match self {
            Self::Nil => LiquidValue::Nil,
            Self::Scalar(value) => value.to_value(),
            Self::Host(value) => value.to_value(),
            Self::InlineError { .. } => LiquidValue::Nil,
            Self::Array(values) => values.to_value(),
            Self::Object(values) => values.to_value(),
            Self::DynamicObject(value) => value.to_value(),
        }
    }

    fn to_live_scope_value(&self) -> LiquidValue {
        match self {
            Self::Nil => LiquidValue::Nil,
            Self::Scalar(value) => value.to_live_scope_value(),
            Self::Host(value) => value.to_live_scope_value(),
            Self::InlineError { .. } => LiquidValue::Nil,
            Self::Array(values) => LiquidValue::Array(
                values
                    .iter()
                    .map(RenderValue::to_live_scope_value)
                    .collect(),
            ),
            Self::Object(values) => LiquidValue::Object(
                values
                    .iter()
                    .map(|(key, value)| (key.clone().into(), value.to_live_scope_value()))
                    .collect(),
            ),
            Self::DynamicObject(value) => value.to_live_scope_value(),
        }
    }

    fn as_scalar(&self) -> Option<ScalarCow<'_>> {
        match self {
            Self::Scalar(value) => value.as_scalar(),
            Self::Host(value) => value.as_scalar(),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        match self {
            Self::Array(values) => Some(values),
            Self::Host(value) => value.as_array(),
            Self::DynamicObject(value) => value.as_array(),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        match self {
            Self::Object(values) => Some(values),
            Self::Host(value) => value.as_object(),
            Self::DynamicObject(value) => Some(value),
            _ => None,
        }
    }

    fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }
}

impl RenderValue {
    fn take_errors(&self) -> Vec<TrackedRenderError> {
        match self {
            Self::Nil | Self::Scalar(_) => Vec::new(),
            Self::Host(value) => value.semantic.take_errors(),
            Self::InlineError { error, .. } => vec![error.clone()],
            Self::Array(values) => values.iter().flat_map(RenderValue::take_errors).collect(),
            Self::Object(values) => take_object_errors(values),
            Self::DynamicObject(value) => value.take_errors(),
        }
    }
}

fn render_error_placeholder(error: &TrackedRenderError) -> String {
    match error {
        TrackedRenderError::Message(message) => {
            if message.starts_with("Liquid error: ") || message.starts_with("Liquid syntax error: ")
            {
                message.clone()
            } else {
                format!("Liquid error: {message}")
            }
        }
        TrackedRenderError::Exception {
            class_name,
            message,
        } => {
            if class_name == "Liquid::SyntaxError" {
                format!("Liquid syntax error: {message}")
            } else {
                format!("Liquid error: {message}")
            }
        }
    }
}

fn take_object_errors(object: &RenderObject) -> Vec<TrackedRenderError> {
    object.values().flat_map(RenderValue::take_errors).collect()
}

pub(crate) fn json_to_object(payload: &str) -> Result<LiquidObject, MagnusError> {
    let parsed: serde_json::Value = serde_json::from_str(payload).map_err(|error| {
        errors::argument_error(
            &magnus::Ruby::get().expect("Ruby VM should be available"),
            format!("invalid JSON payload: {error}"),
        )
    })?;

    liquid::to_object(&parsed).map_err(|error| {
        errors::argument_error(
            &magnus::Ruby::get().expect("Ruby VM should be available"),
            format!("invalid JSON payload for Liquid object conversion: {error}"),
        )
    })
}

pub(crate) fn ruby_to_liquid_value(value: Value) -> Result<LiquidValue, MagnusError> {
    let value = callbacks::call_to_liquid_value(value)?;
    ruby_value_to_liquid_value(value)
}

pub(crate) fn ruby_filter_result_to_liquid_value(value: Value) -> Result<LiquidValue, MagnusError> {
    let semantic = callbacks::call_to_liquid_value(value)?;

    if semantic.is_nil() {
        return Ok(LiquidValue::Nil);
    }

    if let Some(hash) = RHash::from_value(semantic) {
        let mut object = LiquidObject::new();
        hash.foreach(|key: Value, item: Value| {
            let key = ruby_key_to_string(key)?;
            object.insert(key.into(), ruby_filter_result_to_liquid_value(item)?);
            Ok(ForEach::Continue)
        })?;
        return Ok(LiquidValue::Object(object));
    }

    if let Some(array) = RArray::from_value(semantic) {
        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_filter_result_to_liquid_value(item)?);
        }
        return Ok(LiquidValue::Array(items));
    }

    if unsafe { semantic.classname() }.as_ref() == "Float" {
        let float = f64::try_convert(semantic)?;
        return Ok(LiquidValue::scalar(float));
    }

    if !ruby_value_is_direct_scalar(value)
        && RHash::from_value(value).is_none()
        && RArray::from_value(value).is_none()
    {
        return Ok(live_scope_opaque_snapshot(value, Some(semantic)));
    }

    ruby_value_to_liquid_value(semantic)
}

fn ruby_value_to_liquid_value(value: Value) -> Result<LiquidValue, MagnusError> {
    if value.is_nil() {
        return Ok(LiquidValue::Nil);
    }

    if let Some(hash) = RHash::from_value(value) {
        return Ok(LiquidValue::Object(ruby_to_object(hash.as_value())?));
    }

    if let Some(array) = RArray::from_value(value) {
        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_to_liquid_value(item)?);
        }
        return Ok(LiquidValue::array(items));
    }

    let class_name = unsafe { value.classname() };
    match class_name.as_ref() {
        "TrueClass" => return Ok(LiquidValue::scalar(true)),
        "FalseClass" => return Ok(LiquidValue::scalar(false)),
        "Float" => {
            if let Ok(float) = f64::try_convert(value) {
                return Ok(LiquidValue::scalar(float));
            }
        }
        _ => {}
    }

    if let Ok(integer) = i64::try_convert(value) {
        return Ok(LiquidValue::scalar(integer));
    }

    if let Ok(float) = f64::try_convert(value) {
        return Ok(LiquidValue::scalar(float));
    }

    if let Ok(string) = String::try_convert(value) {
        return Ok(LiquidValue::scalar(string));
    }

    if let Ok(symbol) = Symbol::try_convert(value) {
        return Ok(LiquidValue::scalar(symbol.name()?.to_string()));
    }

    Ok(LiquidValue::scalar(value.to_string()))
}

pub(crate) fn ruby_to_render_object(value: Value) -> Result<RenderObject, MagnusError> {
    ruby_to_render_object_with_context(value, LookupMode::Materialized, None)
}

fn ruby_to_render_object_with_context(
    value: Value,
    lookup_mode: LookupMode,
    context: Option<Value>,
) -> Result<RenderObject, MagnusError> {
    if let Some(hash) = RHash::from_value(value) {
        let mut object = RenderObject::new();
        hash.foreach(|key: Value, val: Value| {
            let key = ruby_key_to_string(key)?;
            object.insert(
                key,
                ruby_to_render_value_with_context(val, lookup_mode, context)?,
            );
            Ok(ForEach::Continue)
        })?;
        Ok(object)
    } else {
        Err(errors::argument_error(
            &magnus::Ruby::get().expect("Ruby VM should be available"),
            "expected a Hash-like object",
        ))
    }
}

fn render_scope_from_value(
    value: Value,
    lookup_mode: LookupMode,
    context: Option<Value>,
) -> Result<RenderScope, MagnusError> {
    assign_context_if_supported(value, context)?;
    if value.respond_to("invoke_drop", false)?
        || lookup_mode == LookupMode::TrackMissing
        || hash_has_dynamic_default(value)?
        || hash_contains_proc(value)?
        || (value.respond_to("key?", false)? && value.respond_to("[]", false)?)
    {
        Ok(RenderScope::Dynamic(RenderDynamicObject::new(
            value,
            lookup_mode,
            context,
        )))
    } else {
        Ok(RenderScope::Materialized(
            ruby_to_render_object_with_context(value, lookup_mode, context)?,
        ))
    }
}

fn hash_has_dynamic_default(value: Value) -> Result<bool, MagnusError> {
    let Some(hash) = RHash::from_value(value) else {
        return Ok(false);
    };

    let default_proc: Value = hash.funcall("default_proc", ())?;
    if !default_proc.is_nil() {
        return Ok(true);
    }

    let default_value: Value = hash.funcall("default", ())?;
    Ok(!default_value.is_nil())
}

pub(crate) fn liquid_to_render_object(object: LiquidObject) -> RenderObject {
    object
        .into_iter()
        .map(|(key, value)| (key.to_string(), liquid_to_render_value(value)))
        .collect()
}

fn ruby_to_render_value(value: Value, lookup_mode: LookupMode) -> Result<RenderValue, MagnusError> {
    ruby_to_render_value_with_context(value, lookup_mode, None)
}

fn ruby_to_render_value_with_context(
    value: Value,
    lookup_mode: LookupMode,
    context: Option<Value>,
) -> Result<RenderValue, MagnusError> {
    assign_context_if_supported(value, context)?;
    let preserve_original = !ruby_value_is_direct_scalar(value)
        && RHash::from_value(value).is_none()
        && RArray::from_value(value).is_none();
    let render_override =
        if value.respond_to("to_liquid_value", false)? && !ruby_value_is_direct_scalar(value) {
            Some(value.to_string())
        } else {
            None
        };

    let semantic = callbacks::call_to_liquid_value(value)?;
    assign_context_if_supported(semantic, context)?;

    if semantic.is_nil() {
        return Ok(RenderValue::Nil);
    }

    let semantic_value = if let Some(hash) = RHash::from_value(semantic) {
        RenderValue::DynamicObject(RenderDynamicObject::new(
            hash.as_value(),
            lookup_mode,
            context,
        ))
    } else if unsafe { semantic.classname() }.as_ref() == "Range"
        && semantic.respond_to("each", false)?
    {
        let enumerator: Value = semantic.funcall("to_enum", ("each",))?;
        let array_value: Value = enumerator.funcall("to_a", ())?;
        if let Some(array) = RArray::from_value(array_value) {
            let mut items = Vec::with_capacity(array.len());
            for idx in 0..array.len() {
                let item: Value = array.entry(idx as isize)?;
                items.push(ruby_to_render_value_with_context(
                    item,
                    lookup_mode,
                    context,
                )?);
            }
            RenderValue::Array(items)
        } else {
            RenderValue::Nil
        }
    } else if let Some(array) = RArray::from_value(semantic) {
        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_to_render_value_with_context(
                item,
                lookup_mode,
                context,
            )?);
        }
        RenderValue::Array(items)
    } else if let Some(scalar) = ruby_to_scalar(semantic)? {
        RenderValue::Scalar(RenderScalar::new(scalar, render_override))
    } else if semantic.respond_to("[]", false)? {
        RenderValue::DynamicObject(RenderDynamicObject::new(semantic, lookup_mode, context))
    } else {
        RenderValue::Scalar(RenderScalar::new(
            ScalarCow::new(semantic.to_string()),
            render_override,
        ))
    };

    if preserve_original {
        return Ok(RenderValue::Host(RenderHostValue::new(
            value,
            Some(semantic),
            semantic_value,
        )));
    }

    Ok(semantic_value)
}

fn resolve_dynamic_value(
    scope: Value,
    context: Option<Value>,
    key: &str,
    value: Value,
) -> Result<Value, MagnusError> {
    if unsafe { value.classname() }.as_ref() != "Proc" {
        return Ok(value);
    }

    let arity: i64 = value.funcall("arity", ())?;
    let resolved: Value = if arity == 0 {
        value.funcall("call", ())?
    } else if let Some(context) = context {
        value.funcall("call", (context,))?
    } else {
        return Ok(value);
    };
    if let Some(hash) = RHash::from_value(scope) {
        hash.aset(key, resolved)?;
    } else if scope.respond_to("[]=", false)? {
        let _: Value = scope.funcall("[]=", (key, resolved))?;
    }
    Ok(resolved)
}

pub(crate) fn assign_context_if_supported(
    value: Value,
    context: Option<Value>,
) -> Result<(), MagnusError> {
    let Some(context) = context else {
        return Ok(());
    };

    if value.respond_to("context=", false)? {
        let _: Value = value.funcall("context=", (context,))?;
    }

    Ok(())
}

fn hash_contains_proc(value: Value) -> Result<bool, MagnusError> {
    let Some(hash) = RHash::from_value(value) else {
        return Ok(false);
    };

    let mut contains_proc = false;
    hash.foreach(|_key: Value, val: Value| {
        if unsafe { val.classname() }.as_ref() == "Proc" {
            contains_proc = true;
        }
        Ok(ForEach::Continue)
    })?;
    Ok(contains_proc)
}

fn liquid_to_render_value(value: LiquidValue) -> RenderValue {
    match value {
        LiquidValue::Nil => RenderValue::Nil,
        LiquidValue::Scalar(scalar) => {
            RenderValue::Scalar(RenderScalar::new(scalar.into_owned(), None))
        }
        LiquidValue::Array(values) => {
            RenderValue::Array(values.into_iter().map(liquid_to_render_value).collect())
        }
        LiquidValue::Object(values) => RenderValue::Object(liquid_to_render_object(values)),
        LiquidValue::State(state) => RenderValue::Scalar(RenderScalar::new(
            ScalarCow::new(state.to_kstr().into_owned()),
            None,
        )),
    }
}

pub(crate) fn liquid_to_ruby_value(
    ruby: &magnus::Ruby,
    value: &LiquidValue,
) -> Result<Value, MagnusError> {
    match value {
        LiquidValue::Nil => Ok(ruby.qnil().as_value()),
        LiquidValue::Scalar(scalar) => {
            if let Some(boolean) = scalar.to_bool() {
                Ok(boolean.into_value_with(ruby))
            } else if let Some(integer) = scalar.to_integer() {
                Ok(integer.into_value_with(ruby))
            } else if let Some(float) = scalar.to_float() {
                Ok(float.into_value_with(ruby))
            } else {
                Ok(ruby.str_new(scalar.to_kstr().as_str()).as_value())
            }
        }
        LiquidValue::Array(items) => {
            let array = ruby.ary_new();
            for item in items.iter() {
                array.push(liquid_to_ruby_value(ruby, item)?)?;
            }
            Ok(array.as_value())
        }
        LiquidValue::Object(object) => {
            if let Some(value) = restore_live_scope_ruby_value_from_object(object) {
                return Ok(value);
            }
            let hash = ruby.hash_new();
            for (key, item) in object.iter() {
                hash.aset(key.as_str(), liquid_to_ruby_value(ruby, item)?)?;
            }
            Ok(hash.as_value())
        }
        LiquidValue::State(state) => Ok(ruby.str_new(state.to_kstr().as_str()).as_value()),
    }
}

pub(crate) fn liquid_model_to_ruby_value(
    ruby: &magnus::Ruby,
    value: &CoreValue,
) -> Result<Value, MagnusError> {
    match value {
        CoreValue::Nil => Ok(ruby.qnil().as_value()),
        CoreValue::Scalar(scalar) => {
            if let Some(boolean) = scalar.to_bool() {
                Ok(boolean.into_value_with(ruby))
            } else if let Some(integer) = scalar.to_integer() {
                Ok(integer.into_value_with(ruby))
            } else if let Some(float) = scalar.to_float() {
                Ok(float.into_value_with(ruby))
            } else {
                Ok(ruby.str_new(scalar.to_kstr().as_str()).as_value())
            }
        }
        CoreValue::Array(items) => {
            let array = ruby.ary_new();
            for item in items.iter() {
                array.push(liquid_model_to_ruby_value(ruby, item)?)?;
            }
            Ok(array.as_value())
        }
        CoreValue::Object(object) => {
            if let Some(value) = restore_live_scope_ruby_value_from_object(object) {
                return Ok(value);
            }
            let hash = ruby.hash_new();
            for (key, item) in object.iter() {
                hash.aset(key.as_str(), liquid_model_to_ruby_value(ruby, item)?)?;
            }
            Ok(hash.as_value())
        }
        CoreValue::State(state) => Ok(ruby.str_new(state.to_kstr().as_str()).as_value()),
    }
}

pub(crate) fn clear_live_scope_ruby_values() {
    LIVE_SCOPE_RUBY_VALUES.with(|values| values.borrow_mut().clear());
}

fn live_scope_opaque_snapshot(original: Value, semantic: Option<Value>) -> LiquidValue {
    let token = format!(
        "ruby-live-scope-{}",
        LIVE_SCOPE_RUBY_VALUE_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    LIVE_SCOPE_RUBY_VALUES.with(|values| {
        values
            .borrow_mut()
            .insert(
                token.clone(),
                LiveScopeRubyValue {
                    original: BoxValue::new(original),
                    semantic: semantic.map(BoxValue::new),
                },
            );
    });

    let mut object = LiquidObject::new();
    object.insert(
        LIVE_SCOPE_OPAQUE_MARKER_KEY.into(),
        LiquidValue::scalar(true),
    );
    object.insert(
        LIVE_SCOPE_OPAQUE_TOKEN_KEY.into(),
        LiquidValue::scalar(token),
    );
    LiquidValue::Object(object)
}

fn restore_live_scope_ruby_value_from_object(object: &LiquidObject) -> Option<Value> {
    // Only specially tagged marker objects are converted back into live Ruby
    // values; ordinary Liquid objects keep their materialized form.
    let marker = object.get(LIVE_SCOPE_OPAQUE_MARKER_KEY)?;
    if marker.as_scalar()?.to_bool() != Some(true) {
        return None;
    }

    let token = object
        .get(LIVE_SCOPE_OPAQUE_TOKEN_KEY)?
        .to_kstr()
        .into_owned();
    LIVE_SCOPE_RUBY_VALUES.with(|values| {
        let values = values.borrow();
        let entry = values.get(token.as_str())?;
        restore_live_scope_ruby_value(entry)
    })
}

fn restore_live_scope_ruby_value(entry: &LiveScopeRubyValue) -> Option<Value> {
    let original = *entry.original;
    let Some(semantic) = entry.semantic.as_ref().map(|value| **value) else {
        return Some(original);
    };

    let ruby = magnus::Ruby::get().expect("Ruby VM should be available");
    let liquid: RModule = ruby.class_object().const_get("Liquid").ok()?;
    let rust_extension: RModule = liquid.const_get("RustExtension").ok()?;
    let proxy: RClass = rust_extension.const_get("LiveScopeValueProxy").ok()?;
    proxy
        .new_instance((original, semantic))
        .map(ReprValue::as_value)
        .ok()
        .or(Some(original))
}

fn ruby_key_to_string(value: Value) -> Result<String, MagnusError> {
    if let Ok(string) = String::try_convert(value) {
        Ok(string)
    } else if let Ok(symbol) = Symbol::try_convert(value) {
        Ok(symbol.name()?.to_string())
    } else {
        Ok(value.to_string())
    }
}

fn value_contains_contextual_descendants(
    value: Value,
    visited: &mut HashSet<i64>,
) -> Result<bool, MagnusError> {
    if value.respond_to("context=", false)? {
        return Ok(true);
    }

    let raw: i64 = value.funcall("object_id", ())?;
    if !visited.insert(raw) {
        return Ok(false);
    }

    if let Some(hash) = RHash::from_value(value) {
        let mut found = false;
        hash.foreach(|_key: Value, item: Value| {
            if value_contains_contextual_descendants(item, visited)? {
                found = true;
            }
            Ok(ForEach::Continue)
        })?;
        return Ok(found);
    }

    if let Some(array) = RArray::from_value(value) {
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            if value_contains_contextual_descendants(item, visited)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn ruby_value_is_direct_scalar(value: Value) -> bool {
    if value.is_nil() {
        return true;
    }

    if let Some(_hash) = RHash::from_value(value) {
        return false;
    }

    if let Some(_array) = RArray::from_value(value) {
        return false;
    }

    ruby_to_scalar(value).ok().flatten().is_some()
}

fn ruby_to_scalar(value: Value) -> Result<Option<ScalarCow<'static>>, MagnusError> {
    let class_name = unsafe { value.classname() };
    match class_name.as_ref() {
        "TrueClass" => return Ok(Some(ScalarCow::new(true))),
        "FalseClass" => return Ok(Some(ScalarCow::new(false))),
        "Float" => {
            if let Ok(float) = f64::try_convert(value) {
                return Ok(Some(ScalarCow::new(float)));
            }
        }
        _ => {}
    }

    if let Ok(integer) = i64::try_convert(value) {
        return Ok(Some(ScalarCow::new(integer)));
    }

    if let Ok(float) = f64::try_convert(value) {
        return Ok(Some(ScalarCow::new(float)));
    }

    if let Ok(string) = String::try_convert(value) {
        return Ok(Some(ScalarCow::new(string)));
    }

    if let Ok(symbol) = Symbol::try_convert(value) {
        return Ok(Some(ScalarCow::new(symbol.name()?.to_string())));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use liquid::ParserBuilder;

    use super::json_to_object;

    #[test]
    fn json_payload_renders_simple_variable() {
        let template = ParserBuilder::with_stdlib()
            .build()
            .unwrap()
            .parse("{{test}}")
            .unwrap();
        let globals = json_to_object(r#"{"test":"worked"}"#).unwrap();

        let rendered = template.render(&globals).unwrap();

        assert_eq!(rendered, "worked");
    }
}
