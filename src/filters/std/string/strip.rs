use liquid_compiler::Filter;
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_value::Value;

/// Removes all whitespace (tabs, spaces, and newlines) from both the left and right side of a
/// string.
///
/// It does not affect spaces between words.  Note that while this works for the case of tabs,
/// spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived Core
/// Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "strip",
    description = "Removes all whitespace (tabs, spaces, and newlines) from both the left and right side of a string.",
    parsed(StripFilter)
)]
pub struct Strip;

#[derive(Debug, Default, Display_filter)]
#[name = "strip"]
struct StripFilter;

impl Filter for StripFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let input = input.to_str();
        Ok(Value::scalar(input.trim().to_owned()))
    }
}

/// Removes all whitespaces (tabs, spaces, and newlines) from the beginning of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "lstrip",
    description = "Removes all whitespaces (tabs, spaces, and newlines) from the beginning of a string.",
    parsed(LstripFilter)
)]
pub struct Lstrip;

#[derive(Debug, Default, Display_filter)]
#[name = "lstrip"]
struct LstripFilter;

impl Filter for LstripFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let input = input.to_str();
        Ok(Value::scalar(input.trim_left().to_owned()))
    }
}

/// Removes all whitespace (tabs, spaces, and newlines) from the right side of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "rstrip",
    description = "Removes all whitespace (tabs, spaces, and newlines) from the right side of a string.",
    parsed(RstripFilter)
)]
pub struct Rstrip;

#[derive(Debug, Default, Display_filter)]
#[name = "rstrip"]
struct RstripFilter;

impl Filter for RstripFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let input = input.to_str();
        Ok(Value::scalar(input.trim_right().to_owned()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "strip_newlines",
    description = "Removes any newline characters (line breaks) from a string.",
    parsed(StripNewlinesFilter)
)]
pub struct StripNewlines;

#[derive(Debug, Default, Display_filter)]
#[name = "strip_newlines"]
struct StripNewlinesFilter;

impl Filter for StripNewlinesFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let input = input.to_str();
        Ok(Value::scalar(
            input
                .chars()
                .filter(|c| *c != '\n' && *c != '\r')
                .collect::<String>(),
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
    fn unit_lstrip() {
        let input = &tos!(" 	 \n \r test");
        let desired_result = tos!("test");
        assert_eq!(unit!(Lstrip, input), desired_result);
    }

    #[test]
    fn unit_lstrip_non_string() {
        let input = &Value::scalar(0f64);
        let desired_result = tos!("0");
        assert_eq!(unit!(Lstrip, input), desired_result);
    }

    #[test]
    fn unit_lstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        failed!(Lstrip, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_lstrip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/lstrip/
        let input = &tos!("          So much room for activities!          ");
        let desired_result = tos!("So much room for activities!          ");
        assert_eq!(unit!(Lstrip, input), desired_result);
    }

    #[test]
    fn unit_lstrip_trailing_sequence() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let desired_result = tos!("test 	 \n \r ");
        assert_eq!(unit!(Lstrip, input), desired_result);
    }

    #[test]
    fn unit_lstrip_trailing_sequence_only() {
        let input = &tos!("test 	 \n \r ");
        let desired_result = tos!("test 	 \n \r ");
        assert_eq!(unit!(Lstrip, input), desired_result);
    }

    #[test]
    fn unit_rstrip() {
        let input = &tos!("test 	 \n \r ");
        let desired_result = tos!("test");
        assert_eq!(unit!(Rstrip, input), desired_result);
    }

    #[test]
    fn unit_rstrip_leading_sequence() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let desired_result = tos!(" 	 \n \r test");
        assert_eq!(unit!(Rstrip, input), desired_result);
    }

    #[test]
    fn unit_rstrip_leading_sequence_only() {
        let input = &tos!(" 	 \n \r test");
        let desired_result = tos!(" 	 \n \r test");
        assert_eq!(unit!(Rstrip, input), desired_result);
    }

    #[test]
    fn unit_rstrip_non_string() {
        let input = &Value::scalar(0f64);
        let desired_result = tos!("0");
        assert_eq!(unit!(Rstrip, input), desired_result);
    }

    #[test]
    fn unit_rstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        failed!(Rstrip, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_rstrip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/rstrip/
        let input = &tos!("          So much room for activities!          ");
        let desired_result = tos!("          So much room for activities!");
        assert_eq!(unit!(Rstrip, input), desired_result);
    }

    #[test]
    fn unit_strip() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let desired_result = tos!("test");
        assert_eq!(unit!(Strip, input), desired_result);
    }

    #[test]
    fn unit_strip_leading_sequence_only() {
        let input = &tos!(" 	 \n \r test");
        let desired_result = tos!("test");
        assert_eq!(unit!(Strip, input), desired_result);
    }

    #[test]
    fn unit_strip_non_string() {
        let input = &Value::scalar(0f64);
        let desired_result = tos!("0");
        assert_eq!(unit!(Strip, input), desired_result);
    }

    #[test]
    fn unit_strip_one_argument() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        failed!(Strip, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_strip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/strip/
        let input = &tos!("          So much room for activities!          ");
        let desired_result = tos!("So much room for activities!");
        assert_eq!(unit!(Strip, input), desired_result);
    }

    #[test]
    fn unit_strip_trailing_sequence_only() {
        let input = &tos!("test 	 \n \r ");
        let desired_result = tos!("test");
        assert_eq!(unit!(Strip, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines() {
        let input = &tos!("a\nb\n");
        let desired_result = tos!("ab");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines_between_only() {
        let input = &tos!("a\nb");
        let desired_result = tos!("ab");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines_leading_only() {
        let input = &tos!("\nab");
        let desired_result = tos!("ab");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines_non_string() {
        let input = &Value::scalar(0f64);
        let desired_result = tos!("0");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines_one_argument() {
        let input = &tos!("ab\n");
        failed!(StripNewlines, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_strip_newlines_shopify_liquid() {
        // Test from https://shopify.github.io/liquid/filters/strip_newlines/
        let input = &tos!("\nHello\nthere\n");
        let desired_result = tos!("Hellothere");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }

    #[test]
    fn unit_strip_newlines_trailing_only() {
        let input = &tos!("ab\n");
        let desired_result = tos!("ab");
        assert_eq!(unit!(StripNewlines, input), desired_result);
    }
}
