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

#[derive(Clone, Debug)]
pub(crate) enum RenderValue {
    Nil,
    Scalar(RenderScalar),
    Array(Vec<RenderValue>),
    Object(RenderObject),
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
        }
    }

    fn source(&self) -> DisplayCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.source(),
            Self::Scalar(value) => value.source(),
            Self::Array(values) => values.source(),
            Self::Object(values) => values.source(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => LiquidValue::Nil.type_name(),
            Self::Scalar(value) => value.type_name(),
            Self::Array(values) => values.type_name(),
            Self::Object(values) => values.type_name(),
        }
    }

    fn query_state(&self, state: State) -> bool {
        match self {
            Self::Nil => LiquidValue::Nil.query_state(state),
            Self::Scalar(value) => value.query_state(state),
            Self::Array(values) => values.query_state(state),
            Self::Object(values) => values.query_state(state),
        }
    }

    fn to_kstr(&self) -> KStringCow<'_> {
        match self {
            Self::Nil => LiquidValue::Nil.to_kstr(),
            Self::Scalar(value) => value.to_kstr(),
            Self::Array(values) => values.to_kstr(),
            Self::Object(values) => values.to_kstr(),
        }
    }

    fn to_value(&self) -> LiquidValue {
        match self {
            Self::Nil => LiquidValue::Nil,
            Self::Scalar(value) => value.to_value(),
            Self::Array(values) => values.to_value(),
            Self::Object(values) => values.to_value(),
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
            _ => None,
        }
    }

    fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }
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
            object.insert(key, ruby_to_render_value(val)?);
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

pub(crate) fn liquid_to_render_object(object: LiquidObject) -> RenderObject {
    object
        .into_iter()
        .map(|(key, value)| (key.to_string(), liquid_to_render_value(value)))
        .collect()
}

fn ruby_to_render_value(value: Value) -> Result<RenderValue, MagnusError> {
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
        return Ok(RenderValue::Object(ruby_to_render_object(hash.as_value())?));
    }

    if let Some(array) = RArray::from_value(semantic) {
        let mut items = Vec::with_capacity(array.len());
        for idx in 0..array.len() {
            let item: Value = array.entry(idx as isize)?;
            items.push(ruby_to_render_value(item)?);
        }
        return Ok(RenderValue::Array(items));
    }

    if let Some(scalar) = ruby_to_scalar(semantic)? {
        return Ok(RenderValue::Scalar(RenderScalar::new(scalar, render_override)));
    }

    Ok(RenderValue::Scalar(RenderScalar::new(
        ScalarCow::new(semantic.to_string()),
        render_override,
    )))
}

fn liquid_to_render_value(value: LiquidValue) -> RenderValue {
    match value {
        LiquidValue::Nil => RenderValue::Nil,
        LiquidValue::Scalar(scalar) => RenderValue::Scalar(RenderScalar::new(scalar.into_owned(), None)),
        LiquidValue::Array(values) => {
            RenderValue::Array(values.into_iter().map(liquid_to_render_value).collect())
        }
        LiquidValue::Object(values) => RenderValue::Object(liquid_to_render_object(values)),
        LiquidValue::State(state) => {
            RenderValue::Scalar(RenderScalar::new(ScalarCow::new(state.to_kstr().into_owned()), None))
        }
    }
}

fn json_to_liquid_value(value: serde_json::Value) -> Result<LiquidValue, MagnusError> {
    Ok(match value {
        serde_json::Value::Null => LiquidValue::Nil,
        serde_json::Value::Bool(value) => LiquidValue::scalar(value),
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                LiquidValue::scalar(value)
            } else if let Some(value) = number.as_f64() {
                LiquidValue::scalar(value)
            } else {
                LiquidValue::scalar(number.to_string())
            }
        }
        serde_json::Value::String(value) => LiquidValue::scalar(value),
        serde_json::Value::Array(values) => {
            LiquidValue::array(values.into_iter().map(json_to_liquid_value).collect::<Result<Vec<_>, _>>()?)
        }
        serde_json::Value::Object(values) => {
            let mut object = LiquidObject::new();
            for (key, value) in values {
                object.insert(key.into(), json_to_liquid_value(value)?);
            }
            LiquidValue::Object(object)
        }
    })
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
