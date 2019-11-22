use liquid_compiler::Filter;
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_value::Value;

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "downcase",
    description = "Makes each character in a string downcase.",
    parsed(DowncaseFilter)
)]
pub struct Downcase;

#[derive(Debug, Default, Display_filter)]
#[name = "downcase"]
struct DowncaseFilter;

impl Filter for DowncaseFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let s = input.to_str();
        Ok(Value::scalar(s.to_lowercase()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "upcase",
    description = "Makes each character in a string uppercase.",
    parsed(UpcaseFilter)
)]
pub struct Upcase;

#[derive(Debug, Default, Display_filter)]
#[name = "upcase"]
struct UpcaseFilter;

impl Filter for UpcaseFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let s = input.to_str();
        Ok(Value::scalar(s.to_uppercase()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "capitalize",
    description = "Makes the first character of a string capitalized.",
    parsed(CapitalizeFilter)
)]
pub struct Capitalize;

#[derive(Debug, Default, Display_filter)]
#[name = "capitalize"]
struct CapitalizeFilter;

impl Filter for CapitalizeFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let s = input.to_str().to_owned();
        let mut chars = s.chars();
        let capitalized = match chars.next() {
            Some(first_char) => first_char.to_uppercase().chain(chars).collect(),
            None => String::new(),
        };

        Ok(Value::scalar(capitalized))
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
    fn unit_capitalize() {
        assert_eq!(unit!(Capitalize, tos!("abc")), tos!("Abc"));
        assert_eq!(
            unit!(Capitalize, tos!("hello world 21")),
            tos!("Hello world 21")
        );

        // sure that Umlauts work
        assert_eq!(
            unit!(Capitalize, tos!("über ètat, y̆es?")),
            tos!("Über ètat, y̆es?")
        );

        // Weird UTF-8 White space is kept – this is a no-break whitespace!
        assert_eq!(
            unit!(Capitalize, tos!("hello world​")),
            tos!("Hello world​")
        );

        // The uppercase version of some character are more than one character long
        assert_eq!(unit!(Capitalize, tos!("ßß")), tos!("SSß"));
    }

    #[test]
    fn unit_downcase() {
        assert_eq!(unit!(Downcase, tos!("Abc")), tos!("abc"));
        assert_eq!(
            unit!(Downcase, tos!("Hello World 21")),
            tos!("hello world 21")
        );
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(Upcase, tos!("abc")), tos!("ABC"));
        assert_eq!(
            unit!(Upcase, tos!("Hello World 21")),
            tos!("HELLO WORLD 21")
        );
    }
}
