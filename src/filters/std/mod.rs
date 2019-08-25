use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::Value;

mod array;
mod date;
mod html;
mod math;
mod slice;
mod string;
mod url;

pub use self::array::{
    Compact, Concat, First, Join, Last, Map, Reverse, Sort, SortNatural, Uniq, Where,
};
pub use self::date::Date;
pub use self::html::{Escape, EscapeOnce, NewlineToBr, StripHtml};
pub use self::math::{
    Abs, AtLeast, AtMost, Ceil, DividedBy, Floor, Minus, Modulo, Plus, Round, Times,
};
pub use self::slice::Slice;
pub use self::string::case::{Capitalize, Downcase, Upcase};
pub use self::string::operate::{Append, Prepend, Remove, RemoveFirst, Replace, ReplaceFirst};
pub use self::string::strip::{Lstrip, Rstrip, Strip, StripNewlines};
pub use self::string::truncate::{Truncate, TruncateWords};
pub use self::string::Split;
pub use self::url::{UrlDecode, UrlEncode};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "size",
    description = "Returns the size of the input. For an array or object this is the number of elemets. For other values it's the lenght of its string representation.",
    parsed(SizeFilter)
)]
pub struct Size;

#[derive(Debug, Default, Display_filter)]
#[name = "size"]
struct SizeFilter;

impl Filter for SizeFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        match *input {
            Value::Scalar(ref x) => Ok(Value::scalar(x.to_str().len() as i32)),
            Value::Array(ref x) => Ok(Value::scalar(x.len() as i32)),
            Value::Object(ref x) => Ok(Value::scalar(x.len() as i32)),
            _ => Ok(Value::scalar(0i32)),
        }
    }
}

#[derive(Debug, FilterParameters)]
struct DefaultArgs {
    #[parameter(description = "The default value.")]
    default: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "default",
    description = "Sets a default value for the given input.",
    parameters(DefaultArgs),
    parsed(DefaultFilter)
)]
pub struct Default;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "default"]
struct DefaultFilter {
    #[parameters]
    args: DefaultArgs,
}

impl Filter for DefaultFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        if input.is_default() {
            Ok(args.default.clone())
        } else {
            Ok(input.clone())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use liquid_value::Object;

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
    fn unit_size() {
        assert_eq!(unit!(Size, tos!("abc")), Value::scalar(3f64));
        assert_eq!(
            unit!(Size, tos!("this has 22 characters")),
            Value::scalar(22f64)
        );
        assert_eq!(
            unit!(
                Size,
                Value::Array(vec![
                    Value::scalar(0f64),
                    Value::scalar(1f64),
                    Value::scalar(2f64),
                    Value::scalar(3f64),
                    Value::scalar(4f64),
                ])
            ),
            Value::scalar(5f64)
        );
    }

    #[test]
    fn unit_default() {
        assert_eq!(unit!(Default, tos!(""), tos!("bar")), tos!("bar"));
        assert_eq!(unit!(Default, tos!("foo"), tos!("bar")), tos!("foo"));
        assert_eq!(
            unit!(Default, Value::scalar(0_f64), tos!("bar")),
            Value::scalar(0_f64)
        );
        assert_eq!(
            unit!(Default, Value::Array(vec![]), Value::scalar(1_f64)),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(Default, Value::Array(vec![tos!("")]), Value::scalar(1_f64)),
            Value::Array(vec![tos!("")])
        );
        assert_eq!(
            unit!(Default, Value::Object(Object::new()), Value::scalar(1_f64)),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(Default, Value::scalar(false), Value::scalar(1_f64)),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(Default, Value::scalar(true), Value::scalar(1_f64)),
            Value::scalar(true)
        );
    }
}
