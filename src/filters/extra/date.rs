use chrono::FixedOffset;
use filters::invalid_input;
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::{Scalar, Value};

// liquid-rust proprietary

#[derive(Debug, FilterParameters)]
struct DateInTzArgs {
    #[parameter(description = "The format to return the date in.", arg_type = "str")]
    format: Expression,
    #[parameter(
        description = "The timezone to convert the date to.",
        arg_type = "integer"
    )]
    timezone: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "date_in_tz",
    description = "Converts a timestamp into another date format and timezone.",
    parameters(DateInTzArgs),
    parsed(DateInTzFilter)
)]
pub struct DateInTz;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "date_in_tz"]
struct DateInTzFilter {
    #[parameters]
    args: DateInTzArgs,
}

impl Filter for DateInTzFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let date = input
            .as_scalar()
            .and_then(Scalar::to_date)
            .ok_or_else(|| invalid_input("Invalid date format"))?;

        let timezone = FixedOffset::east(args.timezone * 3600);

        Ok(Value::scalar(
            date.with_timezone(&timezone)
                .format(args.format.as_ref())
                .to_string(),
        ))
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
    fn unit_date_in_tz_same_day() {
        let input = tos!("13 Jun 2016 12:00:00 +0000");
        let unit_result = unit!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(3i32)
        );
        let desired_result = tos!("2016-06-13 15:00:00 +0300");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_date_in_tz_previous_day() {
        let input = tos!("13 Jun 2016 12:00:00 +0000");
        let unit_result = unit!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(-13i32)
        );
        let desired_result = tos!("2016-06-12 23:00:00 -1300");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_date_in_tz_next_day() {
        let input = tos!("13 Jun 2016 12:00:00 +0000");
        let unit_result = unit!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(13i32)
        );
        let desired_result = tos!("2016-06-14 01:00:00 +1300");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_date_in_tz_input_not_a_string() {
        let input = &Value::scalar(0f64);
        failed!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(0i32)
        );
    }

    #[test]
    fn unit_date_in_tz_input_not_a_date_string() {
        let input = &tos!("blah blah blah");
        failed!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(0i32)
        );
    }

    #[test]
    fn unit_date_in_tz_offset_not_a_num() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        failed!(DateInTz, input, tos!("%Y-%m-%d %H:%M:%S %z"), tos!("Hello"));
    }

    #[test]
    fn unit_date_in_tz_zero_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        failed!(DateInTz, input);
    }

    #[test]
    fn unit_date_in_tz_one_argument() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        failed!(DateInTz, input, tos!("%Y-%m-%d %H:%M:%S %z"));
    }

    #[test]
    fn unit_date_in_tz_three_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        failed!(
            DateInTz,
            input,
            tos!("%Y-%m-%d %H:%M:%S %z"),
            Value::scalar(0f64),
            Value::scalar(1f64)
        );
    }
}
