use filters::invalid_input;
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::{Scalar, Value};

// shopify-specific

#[derive(Debug, FilterParameters)]
struct PluralizeArgs {
    #[parameter(description = "The singular version of the string.")]
    singular: Expression,
    #[parameter(description = "The plural version of the string.")]
    plural: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "pluralize",
    description = "Outputs the singular or plural version of a string based on the value of the input.",
    parameters(PluralizeArgs),
    parsed(PluralizeFilter)
)]
pub struct Pluralize;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "pluralize"]
struct PluralizeFilter {
    #[parameters]
    args: PluralizeArgs,
}

impl Filter for PluralizeFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let n = input
            .as_scalar()
            .and_then(Scalar::to_integer)
            .ok_or_else(|| invalid_input("Whole number expected"))?;

        if (n as isize) == 1 {
            Ok(args.singular.clone())
        } else {
            Ok(args.plural.clone())
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
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            let filter = ::liquid::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_pluralize() {
        assert_eq!(
            unit!(Pluralize, Value::scalar(1i32), tos!("one"), tos!("many")),
            tos!("one")
        );

        assert_eq!(
            unit!(Pluralize, Value::scalar(2i32), tos!("one"), tos!("many")),
            tos!("many")
        );
    }
}
