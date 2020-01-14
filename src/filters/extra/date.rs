use chrono::FixedOffset;
use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

use crate::filters::invalid_input;

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
            .and_then(|s| s.to_date_time())
            .ok_or_else(|| invalid_input("Invalid date format"))?;

        let timezone = FixedOffset::east(args.timezone * 3600);

        let formatter = date.with_timezone(&timezone).format(args.format.as_str());
        let date = formatter.to_string();
        Ok(Value::scalar(date))
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
            let positional = Box::new(vec![$(::liquid_core::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid_core::compiler::FilterArguments { positional, keyword };

            let context = ::liquid_core::interpreter::Context::default();

            let filter = ::liquid_core::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid_core::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid_core::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid_core::compiler::FilterArguments { positional, keyword };

            let context = ::liquid_core::interpreter::Context::default();

            ::liquid_core::compiler::ParseFilter::parse(&$a, args)
                .and_then(|filter| ::liquid_core::compiler::Filter::evaluate(&*filter, &$b, &context))
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
