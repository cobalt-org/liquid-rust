use liquid::model::Value as LiquidValue;
use liquid::Object as LiquidObject;
use liquid::ValueView;
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
