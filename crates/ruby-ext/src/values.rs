use std::cell::RefCell;
use std::collections::HashMap;

use liquid::model::{ArrayView, DisplayCow, KStringCow, ScalarCow, State, Value as LiquidValue};
use liquid::{Object as LiquidObject, ObjectView, ValueView};
use magnus::{
    r_array::RArray,
    r_hash::{ForEach, RHash},
    value::ReprValue,
    Error as MagnusError, Symbol, TryConvert, Value,
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

#[derive(Debug)]
pub(crate) struct RenderRootObject {
    scopes: Vec<RenderScope>,
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
        Ok(Self { scopes })
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

    pub(crate) fn take_errors(&self) -> Vec<String> {
        self.scopes
            .iter()
            .flat_map(RenderScope::take_errors)
            .collect()
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
        self.scopes.iter().rev().find_map(|scope| scope.get(index))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RenderValue {
    Nil,
    Scalar(RenderScalar),
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

#[derive(Debug)]
pub(crate) struct RenderDynamicObject {
    value: Value,
    context: Option<Value>,
    cache: RefCell<HashMap<String, Box<RenderValue>>>,
    errors: RefCell<Vec<String>>,
    lookup_mode: LookupMode,
}

impl Clone for RenderDynamicObject {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            context: self.context,
            cache: RefCell::new(HashMap::new()),
            errors: RefCell::new(Vec::new()),
            lookup_mode: self.lookup_mode,
        }
    }
}

impl RenderDynamicObject {
    fn new(value: Value, lookup_mode: LookupMode, context: Option<Value>) -> Self {
        Self {
            value,
            context,
            cache: RefCell::new(HashMap::new()),
            errors: RefCell::new(Vec::new()),
            lookup_mode,
        }
    }

    fn render_string(&self) -> String {
        self.value.to_string()
    }

    fn lookup_value(&self, index: &str) -> Result<Option<Value>, MagnusError> {
        let mut present_via_key_check = false;

        if !hash_has_dynamic_default(self.value)? && self.value.respond_to("key?", false)? {
            let has_key: bool = self.value.funcall("key?", (index,))?;
            if !has_key {
                self.record_missing(index);
                return Ok(None);
            }
            present_via_key_check = true;
        }

        if self.value.respond_to("[]", false)? {
            let result: Value = self.value.funcall("[]", (index,))?;
            if result.is_nil() {
                if present_via_key_check {
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            } else {
                resolve_dynamic_value(self.value, self.context, index, result).map(Some)
            }
        } else {
            self.record_missing(index);
            Ok(None)
        }
    }

    fn record_error(&self, error: MagnusError) {
        self.errors.borrow_mut().push(error.to_string());
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
        self.errors.borrow_mut().push(message);
    }

    fn take_errors(&self) -> Vec<String> {
        let mut errors = std::mem::take(&mut *self.errors.borrow_mut());
        let cache = self.cache.borrow();
        for value in cache.values() {
            errors.extend(value.take_errors());
        }
        errors
    }

    fn hash_keys_owned(&self) -> Vec<String> {
        let Some(hash) = RHash::from_value(self.value) else {
            return Vec::new();
        };

        let mut keys = Vec::new();
        let _ = hash.foreach(|key: Value, _val: Value| {
            keys.push(ruby_key_to_string(key)?);
            Ok(ForEach::Continue)
        });
        keys
    }
}

impl RenderScope {
    fn keys_owned(&self) -> Vec<String> {
        match self {
            Self::Materialized(object) => object.keys().cloned().collect(),
            Self::Dynamic(object) => object.hash_keys_owned(),
        }
    }

    fn contains_key(&self, index: &str) -> bool {
        match self {
            Self::Materialized(object) => object.contains_key(index),
            Self::Dynamic(object) => object.contains_key(index),
        }
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        match self {
            Self::Materialized(object) => object.get(index).map(|value| value as &dyn ValueView),
            Self::Dynamic(object) => object.get(index),
        }
    }

    fn take_errors(&self) -> Vec<String> {
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
        LiquidValue::scalar(self.render_string())
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for RenderDynamicObject {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        RHash::from_value(self.value)
            .map(|hash| hash.len() as i64)
            .unwrap_or(0)
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(
            self.hash_keys_owned()
                .into_iter()
                .map(KStringCow::from_string),
        )
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        Box::new(std::iter::empty())
    }

    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        Box::new(std::iter::empty())
    }

    fn contains_key(&self, index: &str) -> bool {
        match self.lookup_value(index) {
            Ok(None) if self.lookup_mode == LookupMode::TrackMissing => true,
            Ok(value) => value.is_some(),
            Err(error) => {
                self.record_error(error);
                self.lookup_mode == LookupMode::TrackMissing
            }
        }
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        {
            let cache = self.cache.borrow();
            if let Some(value) = cache.get(index) {
                let ptr: *const RenderValue = &**value;
                drop(cache);
                return Some(unsafe { &*ptr } as &dyn ValueView);
            }
        }

        let value = match self.lookup_value(index) {
            Ok(None) if self.lookup_mode == LookupMode::TrackMissing => {
                return Some(&LiquidValue::Nil as &dyn ValueView);
            }
            Ok(value) => value?,
            Err(error) => {
                self.record_error(error);
                if self.lookup_mode == LookupMode::TrackMissing {
                    return Some(&LiquidValue::Nil as &dyn ValueView);
                }
                return None;
            }
        };
        let render_value =
            ruby_to_render_value_with_context(value, self.lookup_mode, self.context).ok()?;

        let mut cache = self.cache.borrow_mut();
        let entry = cache
            .entry(index.to_string())
            .or_insert_with(|| Box::new(render_value));
        let ptr: *const RenderValue = &**entry;
        drop(cache);
        Some(unsafe { &*ptr } as &dyn ValueView)
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
            Self::Array(values) => values.render(),
            Self::Object(values) => values.render(),
            Self::DynamicObject(value) => value.render(),
        }
    }

    fn source(&self) -> DisplayCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.source(),
            Self::Scalar(value) => value.source(),
            Self::Array(values) => values.source(),
            Self::Object(values) => values.source(),
            Self::DynamicObject(value) => value.source(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => LiquidValue::Nil.type_name(),
            Self::Scalar(value) => value.type_name(),
            Self::Array(values) => values.type_name(),
            Self::Object(values) => values.type_name(),
            Self::DynamicObject(value) => value.type_name(),
        }
    }

    fn query_state(&self, state: State) -> bool {
        match self {
            Self::Nil => LiquidValue::Nil.query_state(state),
            Self::Scalar(value) => value.query_state(state),
            Self::Array(values) => values.query_state(state),
            Self::Object(values) => values.query_state(state),
            Self::DynamicObject(value) => value.query_state(state),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.to_kstr(),
            Self::Scalar(value) => value.to_kstr(),
            Self::Array(values) => values.to_kstr(),
            Self::Object(values) => values.to_kstr(),
            Self::DynamicObject(value) => value.to_kstr(),
        }
    }

    fn to_value(&self) -> LiquidValue {
        match self {
            Self::Nil => LiquidValue::Nil,
            Self::Scalar(value) => value.to_value(),
            Self::Array(values) => values.to_value(),
            Self::Object(values) => values.to_value(),
            Self::DynamicObject(value) => value.to_value(),
        }
    }

    fn as_scalar(&self) -> Option<ScalarCow<'_>> {
        match self {
            Self::Scalar(value) => value.as_scalar(),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&dyn ArrayView> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        match self {
            Self::Object(values) => Some(values),
            Self::DynamicObject(value) => Some(value),
            _ => None,
        }
    }

    fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }
}

impl RenderValue {
    fn take_errors(&self) -> Vec<String> {
        match self {
            Self::Nil | Self::Scalar(_) => Vec::new(),
            Self::Array(values) => values.iter().flat_map(RenderValue::take_errors).collect(),
            Self::Object(values) => take_object_errors(values),
            Self::DynamicObject(value) => value.take_errors(),
        }
    }
}

fn take_object_errors(object: &RenderObject) -> Vec<String> {
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
    if let Some(hash) = RHash::from_value(value) {
        let mut object = RenderObject::new();
        hash.foreach(|key: Value, val: Value| {
            let key = ruby_key_to_string(key)?;
            object.insert(key, ruby_to_render_value(val, LookupMode::Materialized)?);
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
        Ok(RenderScope::Materialized(ruby_to_render_object(value)?))
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
    let render_override =
        if value.respond_to("to_liquid_value", false)? && !ruby_value_is_direct_scalar(value) {
            Some(value.to_string())
        } else {
            None
        };

    let semantic = callbacks::call_to_liquid_value(value)?;

    if semantic.is_nil() {
        return Ok(RenderValue::Nil);
    }

    if let Some(hash) = RHash::from_value(semantic) {
        if lookup_mode == LookupMode::TrackMissing || hash_contains_proc(hash.as_value())? {
            return Ok(RenderValue::DynamicObject(RenderDynamicObject::new(
                hash.as_value(),
                lookup_mode,
                context,
            )));
        }

        return Ok(RenderValue::Object(ruby_to_render_object(hash.as_value())?));
    }

    if let Some(array) = RArray::from_value(semantic) {
        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_to_render_value_with_context(
                item,
                lookup_mode,
                context,
            )?);
        }
        return Ok(RenderValue::Array(items));
    }

    if let Some(scalar) = ruby_to_scalar(semantic)? {
        return Ok(RenderValue::Scalar(RenderScalar::new(
            scalar,
            render_override,
        )));
    }

    if semantic.respond_to("[]", false)? {
        return Ok(RenderValue::DynamicObject(RenderDynamicObject::new(
            semantic,
            lookup_mode,
            context,
        )));
    }

    Ok(RenderValue::Scalar(RenderScalar::new(
        ScalarCow::new(semantic.to_string()),
        render_override,
    )))
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
        LiquidValue::Scalar(scalar) => Ok(ruby.str_new(scalar.to_kstr().as_str()).as_value()),
        LiquidValue::Array(items) => {
            let array = ruby.ary_new();
            for item in items.iter() {
                array.push(liquid_to_ruby_value(ruby, item)?)?;
            }
            Ok(array.as_value())
        }
        LiquidValue::Object(object) => {
            let hash = ruby.hash_new();
            for (key, item) in object.iter() {
                hash.aset(key.as_str(), liquid_to_ruby_value(ruby, item)?)?;
            }
            Ok(hash.as_value())
        }
        LiquidValue::State(state) => Ok(ruby.str_new(state.to_kstr().as_str()).as_value()),
    }
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
