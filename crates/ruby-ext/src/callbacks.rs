use magnus::{value::ReprValue, Error as MagnusError, Value};

pub(crate) fn call_to_liquid(value: Value) -> Result<Value, MagnusError> {
    if value.respond_to("to_liquid", false)? {
        value.funcall("to_liquid", ())
    } else {
        Ok(value)
    }
}

pub(crate) fn call_to_liquid_value(value: Value) -> Result<Value, MagnusError> {
    if value.respond_to("to_liquid_value", false)? {
        value.funcall("to_liquid_value", ())
    } else {
        call_to_liquid(value)
    }
}

pub(crate) fn call_liquid_method_missing(
    value: Value,
    method_name: &str,
) -> Result<Value, MagnusError> {
    if value.respond_to("liquid_method_missing", false)? {
        value.funcall("liquid_method_missing", (method_name,))
    } else {
        Ok(value)
    }
}
