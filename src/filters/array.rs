use std::fmt::Write;

use liquid_value::Value;

use super::{check_args_len, invalid_input};
use compiler::FilterResult;

/// Receives a `Value::Array` as an input.
/// Returns a copy of the input with the given value appended at the end.
pub fn push(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let value = args[0].clone();
    let mut array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?
        .clone();
    array.push(value);

    Ok(Value::array(array))
}

/// Receives a `Value::Array` as an input.
/// Returns a copy of the input with the last element removed.
pub fn pop(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let mut array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?
        .clone();
    array.pop();

    Ok(Value::array(array))
}

/// Receives a `Value::Array` as an input.
/// Returns a copy of the input with the given value appended at the beginning.
pub fn unshift(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let value = args[0].clone();
    let mut array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?
        .clone();
    array.insert(0, value);

    Ok(Value::array(array))
}

/// Receives a `Value::Array` as an input.
/// Returns a copy of the input with the first element removed.
pub fn shift(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let mut array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?
        .clone();

    if !array.is_empty() {
        array.remove(0);
    }

    Ok(Value::array(array))
}

/// Convert an array into a sentence.
pub fn array_to_sentence_string(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 1)?;

    let connector = args.get(0).map(|arg| arg.to_str());
    let connector = connector
        .as_ref()
        .map(|s| s.as_ref())
        .unwrap_or_else(|| "and");

    let mut array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?
        .iter();

    let mut sentence = array
        .next()
        .map(|v| v.to_str().into_owned())
        .unwrap_or_else(|| "".to_string());

    let last = array.next_back();

    for value in array {
        write!(sentence, ", {}", value.render()).expect("It should be safe to write to a string.");
    }

    if let Some(last) = last {
        write!(sentence, ", {} {}", connector, last.render())
            .expect("It should be safe to write to a string.");
    }

    Ok(Value::scalar(sentence))
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    #[test]
    fn unit_push() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let args = &[Value::scalar("Spokane")];
        let desired_result = Value::Array(vec![
            Value::scalar("Seattle"),
            Value::scalar("Tacoma"),
            Value::scalar("Spokane"),
        ]);
        assert_eq!(unit!(push, input, args), desired_result);
    }

    #[test]
    fn unit_pop() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let args = &[];
        let desired_result = Value::Array(vec![Value::scalar("Seattle")]);
        assert_eq!(unit!(pop, input, args), desired_result);
    }

    #[test]
    fn unit_pop_empty() {
        let input = Value::Array(vec![]);
        let args = &[];
        let desired_result = Value::Array(vec![]);
        assert_eq!(unit!(pop, input, args), desired_result);
    }

    #[test]
    fn unit_unshift() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let args = &[Value::scalar("Olympia")];
        let desired_result = Value::Array(vec![
            Value::scalar("Olympia"),
            Value::scalar("Seattle"),
            Value::scalar("Tacoma"),
        ]);
        assert_eq!(unit!(unshift, input, args), desired_result);
    }

    #[test]
    fn unit_shift() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let args = &[];
        let desired_result = Value::Array(vec![Value::scalar("Tacoma")]);
        assert_eq!(unit!(shift, input, args), desired_result);
    }

    #[test]
    fn unit_shift_empty() {
        let input = Value::Array(vec![]);
        let args = &[];
        let desired_result = Value::Array(vec![]);
        assert_eq!(unit!(shift, input, args), desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string() {
        let input = Value::Array(vec![
            Value::scalar("foo"),
            Value::scalar("bar"),
            Value::scalar("baz"),
        ]);
        let args = &[];
        let desired_result = Value::scalar("foo, bar, and baz");
        assert_eq!(unit!(array_to_sentence_string, input, args), desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_two_elements() {
        let input = Value::Array(vec![Value::scalar("foo"), Value::scalar("bar")]);
        let args = &[];
        let desired_result = Value::scalar("foo, and bar");
        assert_eq!(unit!(array_to_sentence_string, input, args), desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_one_element() {
        let input = Value::Array(vec![Value::scalar("foo")]);
        let args = &[];
        let desired_result = Value::scalar("foo");
        assert_eq!(unit!(array_to_sentence_string, input, args), desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_no_elements() {
        let input = Value::Array(vec![]);
        let args = &[];
        let desired_result = Value::scalar("");
        assert_eq!(unit!(array_to_sentence_string, input, args), desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_custom_connector() {
        let input = Value::Array(vec![
            Value::scalar("foo"),
            Value::scalar("bar"),
            Value::scalar("baz"),
        ]);
        let args = &[Value::scalar("or")];
        let desired_result = Value::scalar("foo, bar, or baz");
        assert_eq!(unit!(array_to_sentence_string, input, args), desired_result);
    }
}
