use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

#[derive(Debug, FilterParameters)]
struct DateArgs {
    #[parameter(description = "The format to return the date in.", arg_type = "str")]
    format: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "date",
    description = "Converts a timestamp into another date format.",
    parameters(DateArgs),
    parsed(DateFilter)
)]
pub struct Date;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "date"]
struct DateFilter {
    #[parameters]
    args: DateArgs,
}

impl Filter for DateFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let date = input.as_scalar().and_then(|s| s.to_date_time());
        match date {
            Some(date) if !args.format.is_empty() => {
                Ok(Value::scalar(date.format(args.format.as_str()).to_string()))
            }
            _ => Ok(input.clone()),
        }
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
    fn unit_date() {
        assert_eq!(
            unit!(Date, tos!("13 Jun 2016 02:30:00 +0300"), tos!("%Y-%m-%d")),
            tos!("2016-06-13")
        );
    }

    #[test]
    fn unit_date_cobalt_format() {
        assert_eq!(
            unit!(Date, tos!("2016-06-13 02:30:00 +0300"), tos!("%Y-%m-%d")),
            tos!("2016-06-13")
        );
    }

    #[test]
    fn unit_date_bad_input_type() {
        assert_eq!(
            unit!(Date, Value::scalar(0f64), tos!("%Y-%m-%d")),
            Value::scalar(0f64)
        );
    }

    #[test]
    fn unit_date_bad_input_format() {
        assert_eq!(
            unit!(Date, tos!("blah blah blah"), tos!("%Y-%m-%d")),
            tos!("blah blah blah")
        );
    }

    #[test]
    fn unit_date_format_empty() {
        assert_eq!(
            unit!(
                Date,
                tos!("13 Jun 2016 02:30:00 +0300"),
                Value::scalar("".to_owned())
            ),
            tos!("13 Jun 2016 02:30:00 +0300")
        );
    }

    #[test]
    fn unit_date_bad_format_type() {
        assert_eq!(
            unit!(
                Date,
                tos!("13 Jun 2016 02:30:00 +0300"),
                Value::scalar(0f64)
            ),
            tos!("0")
        );
    }

    #[test]
    fn unit_date_missing_format() {
        failed!(Date, tos!("13 Jun 2016 02:30:00 +0300"));
    }

    #[test]
    fn unit_date_extra_param() {
        failed!(
            Date,
            tos!("13 Jun 2016 02:30:00 +0300"),
            Value::scalar(0f64),
            Value::scalar(1f64)
        );
    }
}
