use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

pub mod case;
pub mod operate;
pub mod strip;
pub mod truncate;

#[derive(Debug, FilterParameters)]
struct SplitArgs {
    #[parameter(
        description = "The separator between each element in the string.",
        arg_type = "str"
    )]
    pattern: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "split",
    description = "Divides an input string into an array using the argument as a separator.",
    parameters(SplitArgs),
    parsed(SplitFilter)
)]
pub struct Split;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "split"]
struct SplitFilter {
    #[parameters]
    args: SplitArgs,
}

impl Filter for SplitFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input.to_kstr();

        // Split and construct resulting Array
        Ok(Value::Array(
            input
                .split(args.pattern.as_str())
                .map(|s| Value::scalar(s.to_owned()))
                .collect(),
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
    fn unit_split() {
        assert_eq!(
            unit!(Split, tos!("a, b, c"), tos!(", ")),
            Value::Array(vec![tos!("a"), tos!("b"), tos!("c")])
        );
        assert_eq!(
            unit!(Split, tos!("a~b"), tos!("~")),
            Value::Array(vec![tos!("a"), tos!("b")])
        );
    }

    #[test]
    fn unit_split_bad_split_string() {
        let input = tos!("a,b,c");
        let desired_result = Value::Array(vec![tos!("a,b,c")]);
        assert_eq!(unit!(Split, input, Value::scalar(1f64)), desired_result);
    }

    #[test]
    fn unit_split_no_args() {
        let input = tos!("a,b,c");
        failed!(Split, input);
    }
}
