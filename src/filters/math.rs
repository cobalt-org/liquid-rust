use liquid_value::Value;

use super::{check_args_len, invalid_argument, invalid_input};
use interpreter::FilterResult;

/// Returns the absolute value of a number.
pub fn abs(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    match *input {
        Value::Scalar(ref s) => s
            .to_integer()
            .map(|i| Value::scalar(i.abs()))
            .or_else(|| s.to_float().map(|i| Value::scalar(i.abs())))
            .ok_or_else(|| invalid_input("Numeric expected")),
        _ => Err(invalid_input("Number expected")),
    }
}

/// Limits a number to a minimum value.
pub fn at_least(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;
    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let max_value = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| max_value.to_integer().map(|max| Value::scalar(i.max(max))))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| max_value.to_float().map(|max| Value::scalar(i.max(max))))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

/// Limits a number to a maximum value.
pub fn at_most(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;
    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let max_value = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| max_value.to_integer().map(|max| Value::scalar(i.min(max))))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| max_value.to_float().map(|max| Value::scalar(i.min(max))))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

pub fn plus(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let operand = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| operand.to_integer().map(|o| Value::scalar(i + o)))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| operand.to_float().map(|o| Value::scalar(i + o)))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

pub fn minus(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let operand = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| operand.to_integer().map(|o| Value::scalar(i - o)))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| operand.to_float().map(|o| Value::scalar(i - o)))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

pub fn times(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let operand = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| operand.to_integer().map(|o| Value::scalar(i * o)))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| operand.to_float().map(|o| Value::scalar(i * o)))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

pub fn divided_by(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let operand = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| operand.to_integer().map(|o| Value::scalar(i / o)))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| operand.to_float().map(|o| Value::scalar(i / o)))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
}

pub fn modulo(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_scalar()
        .ok_or_else(|| invalid_input("Number expected"))?;

    let operand = args[0]
        .as_scalar()
        .ok_or_else(|| invalid_argument(0, "Number expected"))?;

    let result = input
        .to_integer()
        .and_then(|i| operand.to_integer().map(|o| Value::scalar(i % o)))
        .or_else(|| {
            input
                .to_float()
                .and_then(|i| operand.to_float().map(|o| Value::scalar(i % o)))
        }).ok_or_else(|| invalid_argument(0, "Number expected"))?;

    Ok(result)
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

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap_err()
        }};
    }

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_abs() {
        let input = Value::scalar(-1f64);
        let args = &[];
        let desired_result = Value::scalar(1f64);
        assert_eq!(unit!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_positive_in_string() {
        let input = &tos!("42");
        let args = &[];
        let desired_result = Value::scalar(42f64);
        assert_eq!(unit!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_not_number_or_string() {
        let input = &Value::scalar(true);
        let args = &[];
        failed!(abs, input, args);
    }

    #[test]
    fn unit_abs_one_argument() {
        let input = &Value::scalar(-1f64);
        let args = &[Value::scalar(0f64)];
        failed!(abs, input, args);
    }

    #[test]
    fn unit_abs_shopify_liquid() {
        // Three tests from https://shopify.github.io/liquid/filters/abs/
        assert_eq!(unit!(abs, Value::scalar(-17f64), &[]), Value::scalar(17f64));
        assert_eq!(unit!(abs, Value::scalar(4f64), &[]), Value::scalar(4f64));
        assert_eq!(unit!(abs, tos!("-19.86"), &[]), Value::scalar(19.86f64));
    }
    #[test]
    fn unit_at_least() {
        assert_eq!(
            unit!(at_least, Value::scalar(4f64), &[Value::scalar(5f64)]),
            Value::scalar(5f64)
        );
        assert_eq!(
            unit!(at_least, Value::scalar(4f64), &[Value::scalar(3f64)]),
            Value::scalar(4f64)
        );
        assert_eq!(
            unit!(at_least, Value::scalar(21.5), &[Value::scalar(2.25)]),
            Value::scalar(21.5)
        );
        assert_eq!(
            unit!(at_least, Value::scalar(21.5), &[Value::scalar(42.25)]),
            Value::scalar(42.25)
        );
    }
    #[test]
    fn unit_at_most() {
        assert_eq!(
            unit!(at_most, Value::scalar(4f64), &[Value::scalar(5f64)]),
            Value::scalar(4f64)
        );
        assert_eq!(
            unit!(at_most, Value::scalar(4f64), &[Value::scalar(3f64)]),
            Value::scalar(3f64)
        );
        assert_eq!(
            unit!(at_most, Value::scalar(21.5), &[Value::scalar(2.25)]),
            Value::scalar(2.25)
        );
        assert_eq!(
            unit!(at_most, Value::scalar(21.5), &[Value::scalar(42.25)]),
            Value::scalar(21.5)
        );
    }
    #[test]
    fn unit_plus() {
        assert_eq!(
            unit!(plus, Value::scalar(2f64), &[Value::scalar(1f64)]),
            Value::scalar(3f64)
        );
        assert_eq!(
            unit!(plus, Value::scalar(21.5), &[Value::scalar(2.25)]),
            Value::scalar(23.75)
        );
    }

    #[test]
    fn unit_minus() {
        assert_eq!(
            unit!(minus, Value::scalar(2f64), &[Value::scalar(1f64)]),
            Value::scalar(1f64)
        );
        assert_eq!(
            unit!(minus, Value::scalar(21.5), &[Value::scalar(1.25)]),
            Value::scalar(20.25)
        );
    }

    #[test]
    fn unit_times() {
        assert_eq!(
            unit!(times, Value::scalar(2f64), &[Value::scalar(3f64)]),
            Value::scalar(6f64)
        );
        assert_eq!(
            unit!(times, Value::scalar(8.5), &[Value::scalar(0.5)]),
            Value::scalar(4.25)
        );
        assert!(times(&Value::scalar(true), &[Value::scalar(8.5)]).is_err());
        assert!(times(&Value::scalar(2.5), &[Value::scalar(true)]).is_err());
        assert!(times(&Value::scalar(2.5), &[]).is_err());
    }

    #[test]
    fn unit_modulo() {
        assert_eq!(
            unit!(modulo, Value::scalar(3_f64), &[Value::scalar(2_f64)]),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(modulo, Value::scalar(3_f64), &[Value::scalar(3.0)]),
            Value::scalar(0_f64)
        );
        assert_eq!(
            unit!(modulo, Value::scalar(24_f64), &[Value::scalar(7_f64)]),
            Value::scalar(3_f64)
        );
        assert_eq!(
            unit!(modulo, Value::scalar(183.357), &[Value::scalar(12_f64)]),
            Value::scalar(3.3569999999999993)
        );
    }

    #[test]
    fn unit_divided_by() {
        assert_eq!(
            unit!(divided_by, Value::scalar(4f64), &[Value::scalar(2f64)]),
            Value::scalar(2f64)
        );
        assert_eq!(
            unit!(divided_by, Value::scalar(5f64), &[Value::scalar(2f64)]),
            Value::scalar(2.5f64)
        );
        assert!(divided_by(&Value::scalar(true), &[Value::scalar(8.5)]).is_err());
        assert!(divided_by(&Value::scalar(2.5), &[Value::scalar(true)]).is_err());
        assert!(divided_by(&Value::scalar(2.5), &[]).is_err());
    }
}
