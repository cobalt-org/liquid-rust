use filters::{invalid_argument, invalid_input};
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::{Scalar, Value};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "abs",
    description = "Returns the absolute value of a number.",
    parsed(AbsFilter)
)]
pub struct Abs;

#[derive(Debug, Default, Display_filter)]
#[name = "abs"]
struct AbsFilter;

impl Filter for AbsFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        match *input {
            Value::Scalar(ref s) => s
                .to_integer()
                .map(|i| Value::scalar(i.abs()))
                .or_else(|| s.to_float().map(|i| Value::scalar(i.abs())))
                .ok_or_else(|| invalid_input("Number expected")),
            _ => invalid_input("Number expected").into_err(),
        }
    }
}

#[derive(Debug, FilterParameters)]
struct AtLeastArgs {
    #[parameter(description = "The lower limit of the input.")]
    min: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "at_least",
    description = "Limits a number to a minimum value.",
    parameters(AtLeastArgs),
    parsed(AtLeastFilter)
)]
pub struct AtLeast;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "at_least"]
struct AtLeastFilter {
    #[parameters]
    args: AtLeastArgs,
}

impl Filter for AtLeastFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let min = args
            .min
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| min.to_integer().map(|min| Value::scalar(i.max(min))))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| min.to_float().map(|min| Value::scalar(i.max(min))))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct AtMostArgs {
    #[parameter(description = "The upper limit of the input.")]
    max: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "at_most",
    description = "Limits a number to a maximum value.",
    parameters(AtMostArgs),
    parsed(AtMostFilter)
)]
pub struct AtMost;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "at_most"]
struct AtMostFilter {
    #[parameters]
    args: AtMostArgs,
}

impl Filter for AtMostFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let max = args
            .max
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| max.to_integer().map(|max| Value::scalar(i.min(max))))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| max.to_float().map(|max| Value::scalar(i.min(max))))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct PlusArgs {
    #[parameter(description = "The number to sum to the input.")]
    operand: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "plus",
    description = "Sums a number with the given operand.",
    parameters(PlusArgs),
    parsed(PlusFilter)
)]
pub struct Plus;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "plus"]
struct PlusFilter {
    #[parameters]
    args: PlusArgs,
}

impl Filter for PlusFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let operand = args
            .operand
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| operand.to_integer().map(|o| Value::scalar(i + o)))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| operand.to_float().map(|o| Value::scalar(i + o)))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct MinusArgs {
    #[parameter(description = "The number to subtract to the input.")]
    operand: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "minus",
    description = "Subtracts the given operand from a number.",
    parameters(MinusArgs),
    parsed(MinusFilter)
)]
pub struct Minus;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "minus"]
struct MinusFilter {
    #[parameters]
    args: MinusArgs,
}

impl Filter for MinusFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let operand = args
            .operand
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| operand.to_integer().map(|o| Value::scalar(i - o)))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| operand.to_float().map(|o| Value::scalar(i - o)))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct TimesArgs {
    #[parameter(description = "The number to multiply the input by.")]
    operand: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "times",
    description = "Multiplies a number by the given operand.",
    parameters(TimesArgs),
    parsed(TimesFilter)
)]
pub struct Times;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "times"]
struct TimesFilter {
    #[parameters]
    args: TimesArgs,
}

impl Filter for TimesFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let operand = args
            .operand
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| operand.to_integer().map(|o| Value::scalar(i * o)))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| operand.to_float().map(|o| Value::scalar(i * o)))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct DividedByArgs {
    #[parameter(description = "The number to divide the input by.")]
    operand: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "divided_by",
    description = "Divides a number by the given operand.",
    parameters(DividedByArgs),
    parsed(DividedByFilter)
)]
pub struct DividedBy;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "divided_by"]
struct DividedByFilter {
    #[parameters]
    args: DividedByArgs,
}

impl Filter for DividedByFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let operand = args
            .operand
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| operand.to_integer().map(|o| Value::scalar(i / o)))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| operand.to_float().map(|o| Value::scalar(i / o)))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct ModuloArgs {
    #[parameter(description = "The modulo of the input. Must be a number.")]
    operand: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "modulo",
    description = "The remainder of a division operation of a number by the given operand.",
    parameters(ModuloArgs),
    parsed(ModuloFilter)
)]
pub struct Modulo;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "modulo"]
struct ModuloFilter {
    #[parameters]
    args: ModuloArgs,
}

impl Filter for ModuloFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_scalar()
            .ok_or_else(|| invalid_input("Number expected"))?;

        let operand = args
            .operand
            .as_scalar()
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        let result = input
            .to_integer()
            .and_then(|i| operand.to_integer().map(|o| Value::scalar(i % o)))
            .or_else(|| {
                input
                    .to_float()
                    .and_then(|i| operand.to_float().map(|o| Value::scalar(i % o)))
            })
            .ok_or_else(|| invalid_argument("operand", "Number expected"))?;

        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct RoundArgs {
    #[parameter(
        description = "Number of decimal places. Defaults to 0 (nearest integer).",
        arg_type = "integer"
    )]
    decimal_places: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "round",
    description = "Rounds an input number to the nearest integer or, if a number is specified as an argument, to that number of decimal places.",
    parameters(RoundArgs),
    parsed(RoundFilter)
)]
pub struct Round;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "round"]
struct RoundFilter {
    #[parameters]
    args: RoundArgs,
}

impl Filter for RoundFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let n = args.decimal_places.unwrap_or(0);

        let input = input
            .as_scalar()
            .and_then(Scalar::to_float)
            .ok_or_else(|| invalid_input("Number expected"))?;

        if n == 0 {
            Ok(Value::scalar(input.round() as i32))
        } else if n < 0 {
            Err(invalid_argument(
                "decimal_places",
                "Positive number expected",
            ))
        } else {
            let multiplier = 10.0_f64.powi(n);
            Ok(Value::scalar((input * multiplier).round() / multiplier))
        }
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "ceil",
    description = "Rounds the input up to the nearest whole number.",
    parsed(CeilFilter)
)]
pub struct Ceil;

#[derive(Debug, Default, Display_filter)]
#[name = "ceil"]
struct CeilFilter;

impl Filter for CeilFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let n = input
            .as_scalar()
            .and_then(Scalar::to_float)
            .ok_or_else(|| invalid_input("Number expected"))?;
        Ok(Value::scalar(n.ceil() as i32))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "floor",
    description = "Rounds a number down to the nearest whole number.",
    parsed(FloorFilter)
)]
pub struct Floor;

#[derive(Debug, Default, Display_filter)]
#[name = "floor"]
struct FloorFilter;

impl Filter for FloorFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let n = input
            .as_scalar()
            .and_then(Scalar::to_float)
            .ok_or_else(|| invalid_input("Number expected"))?;
        Ok(Value::scalar(n.floor() as i32))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            let filter = ::liquid::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            ::liquid::compiler::ParseFilter::parse(&$a, args)
                .and_then(|filter| ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context))
                .unwrap_err()
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
        let desired_result = Value::scalar(1f64);
        assert_eq!(unit!(Abs, input), desired_result);
    }

    #[test]
    fn unit_abs_positive_in_string() {
        let input = &tos!("42");
        let desired_result = Value::scalar(42f64);
        assert_eq!(unit!(Abs, input), desired_result);
    }

    #[test]
    fn unit_abs_not_number_or_string() {
        let input = &Value::scalar(true);
        failed!(Abs, input);
    }

    #[test]
    fn unit_abs_one_argument() {
        let input = &Value::scalar(-1f64);
        failed!(Abs, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_abs_shopify_liquid() {
        // Three tests from https://shopify.github.io/liquid/filters/abs/
        assert_eq!(unit!(Abs, Value::scalar(-17f64)), Value::scalar(17f64));
        assert_eq!(unit!(Abs, Value::scalar(4f64)), Value::scalar(4f64));
        assert_eq!(unit!(Abs, tos!("-19.86")), Value::scalar(19.86f64));
    }
    #[test]
    fn unit_at_least() {
        assert_eq!(
            unit!(AtLeast, Value::scalar(4f64), Value::scalar(5f64)),
            Value::scalar(5f64)
        );
        assert_eq!(
            unit!(AtLeast, Value::scalar(4f64), Value::scalar(3f64)),
            Value::scalar(4f64)
        );
        assert_eq!(
            unit!(AtLeast, Value::scalar(21.5), Value::scalar(2.25)),
            Value::scalar(21.5)
        );
        assert_eq!(
            unit!(AtLeast, Value::scalar(21.5), Value::scalar(42.25)),
            Value::scalar(42.25)
        );
    }
    #[test]
    fn unit_at_most() {
        assert_eq!(
            unit!(AtMost, Value::scalar(4f64), Value::scalar(5f64)),
            Value::scalar(4f64)
        );
        assert_eq!(
            unit!(AtMost, Value::scalar(4f64), Value::scalar(3f64)),
            Value::scalar(3f64)
        );
        assert_eq!(
            unit!(AtMost, Value::scalar(21.5), Value::scalar(2.25)),
            Value::scalar(2.25)
        );
        assert_eq!(
            unit!(AtMost, Value::scalar(21.5), Value::scalar(42.25)),
            Value::scalar(21.5)
        );
    }
    #[test]
    fn unit_plus() {
        assert_eq!(
            unit!(Plus, Value::scalar(2f64), Value::scalar(1f64)),
            Value::scalar(3f64)
        );
        assert_eq!(
            unit!(Plus, Value::scalar(21.5), Value::scalar(2.25)),
            Value::scalar(23.75)
        );
    }

    #[test]
    fn unit_minus() {
        assert_eq!(
            unit!(Minus, Value::scalar(2f64), Value::scalar(1f64)),
            Value::scalar(1f64)
        );
        assert_eq!(
            unit!(Minus, Value::scalar(21.5), Value::scalar(1.25)),
            Value::scalar(20.25)
        );
    }

    #[test]
    fn unit_times() {
        assert_eq!(
            unit!(Times, Value::scalar(2f64), Value::scalar(3f64)),
            Value::scalar(6f64)
        );
        assert_eq!(
            unit!(Times, Value::scalar(8.5), Value::scalar(0.5)),
            Value::scalar(4.25)
        );
        failed!(Times, Value::scalar(true), Value::scalar(8.5));
        failed!(Times, Value::scalar(2.5), Value::scalar(true));
        failed!(Times, Value::scalar(2.5));
    }

    #[test]
    fn unit_modulo() {
        assert_eq!(
            unit!(Modulo, Value::scalar(3_f64), Value::scalar(2_f64)),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(Modulo, Value::scalar(3_f64), Value::scalar(3.0)),
            Value::scalar(0_f64)
        );
        assert_eq!(
            unit!(Modulo, Value::scalar(24_f64), Value::scalar(7_f64)),
            Value::scalar(3_f64)
        );
        assert_eq!(
            unit!(Modulo, Value::scalar(183.357), Value::scalar(12_f64)),
            Value::scalar(3.3569999999999993)
        );
    }

    #[test]
    fn unit_divided_by() {
        assert_eq!(
            unit!(DividedBy, Value::scalar(4f64), Value::scalar(2f64)),
            Value::scalar(2f64)
        );
        assert_eq!(
            unit!(DividedBy, Value::scalar(5f64), Value::scalar(2f64)),
            Value::scalar(2.5f64)
        );
        failed!(DividedBy, Value::scalar(true), Value::scalar(8.5));
        failed!(DividedBy, Value::scalar(2.5), Value::scalar(true));
        failed!(DividedBy, Value::scalar(2.5));
    }

    #[test]
    fn unit_ceil() {
        assert_eq!(unit!(Ceil, Value::scalar(1.1f64)), Value::scalar(2f64));
        assert_eq!(unit!(Ceil, Value::scalar(1f64)), Value::scalar(1f64));
        failed!(Ceil, Value::scalar(true));
    }

    #[test]
    fn unit_floor() {
        assert_eq!(unit!(Floor, Value::scalar(1.1f64)), Value::scalar(1f64));
        assert_eq!(unit!(Floor, Value::scalar(1f64)), Value::scalar(1f64));
        failed!(Floor, Value::scalar(true));
    }

    #[test]
    fn unit_round() {
        assert_eq!(unit!(Round, Value::scalar(1.1f64)), Value::scalar(1i32));
        assert_eq!(unit!(Round, Value::scalar(1.5f64)), Value::scalar(2i32));
        assert_eq!(unit!(Round, Value::scalar(2f64)), Value::scalar(2i32));
        failed!(Round, Value::scalar(true));
    }

    #[test]
    fn unit_round_precision() {
        assert_eq!(
            unit!(Round, Value::scalar(1.1f64), Value::scalar(0i32)),
            Value::scalar(1f64)
        );
        assert_eq!(
            unit!(Round, Value::scalar(1.5f64), Value::scalar(1i32)),
            Value::scalar(1.5f64)
        );
        assert_eq!(
            unit!(Round, Value::scalar(3.14159f64), Value::scalar(3i32)),
            Value::scalar(3.142f64)
        );
    }
}
