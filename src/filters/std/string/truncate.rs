use itertools;
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::Value;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, FilterParameters)]
struct TruncateArgs {
    #[parameter(
        description = "The maximum lenght of the string, after which it will be truncated.",
        arg_type = "integer"
    )]
    lenght: Option<Expression>,

    #[parameter(
        description = "The text appended to the end of the string if it is truncated. This text counts to the maximum lenght of the string. Defaults to \"...\".",
        arg_type = "str"
    )]
    ellipsis: Option<Expression>,
}

/// `truncate` shortens a string down to the number of characters passed as a parameter.
///
/// Note that this function operates on [grapheme
/// clusters](http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries) (or *user-perceived
/// character*), rather than Unicode code points.  Each grapheme cluster may be composed of more
/// than one Unicode code point, and does not necessarily correspond to rust's conception of a
/// character.
///
/// If the number of characters specified is less than the length of the string, an ellipsis
/// (`...`) is appended to the string and is included in the character count.
///
/// ## Custom ellipsis
///
/// `truncate` takes an optional second parameter that specifies the sequence of characters to be
/// appended to the truncated string. By default this is an ellipsis (`...`), but you can specify a
/// different sequence.
///
/// The length of the second parameter counts against the number of characters specified by the
/// first parameter. For example, if you want to truncate a string to exactly 10 characters, and
/// use a 3-character ellipsis, use 13 for the first parameter of `truncate`, since the ellipsis
/// counts as 3 characters.
///
/// ## No ellipsis
///
/// You can truncate to the exact number of characters specified by the first parameter and show no
/// trailing characters by passing a blank string as the second parameter.
#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "truncate",
    description = "Shortens a string down to the number of characters passed as a parameter.",
    parameters(TruncateArgs),
    parsed(TruncateFilter)
)]
pub struct Truncate;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "truncate"]
struct TruncateFilter {
    #[parameters]
    args: TruncateArgs,
}

impl Filter for TruncateFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let lenght = args.lenght.unwrap_or(50) as usize;

        let truncate_string = args.ellipsis.unwrap_or_else(|| "...".into());

        let l = cmp::max(lenght - truncate_string.len(), 0);

        let input_string = input.to_kstr();

        let result = if lenght < input_string.len() {
            let result = UnicodeSegmentation::graphemes(input_string.as_str(), true)
                .take(l)
                .collect::<Vec<&str>>()
                .join("")
                .to_string()
                + truncate_string.as_str();
            Value::scalar(result)
        } else {
            input.clone()
        };
        Ok(result)
    }
}

#[derive(Debug, FilterParameters)]
struct TruncateWordsArgs {
    #[parameter(
        description = "The maximum number of words, after which the string will be truncated.",
        arg_type = "integer"
    )]
    lenght: Option<Expression>,

    #[parameter(
        description = "The text appended to the end of the string if it is truncated. This text counts to the maximum word-count of the string. Defaults to \"...\".",
        arg_type = "str"
    )]
    ellipsis: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "truncatewords",
    description = "Shortens a string down to the number of characters passed as a parameter.",
    parameters(TruncateWordsArgs),
    parsed(TruncateWordsFilter)
)]
pub struct TruncateWords;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "truncate"]
struct TruncateWordsFilter {
    #[parameters]
    args: TruncateWordsArgs,
}

impl Filter for TruncateWordsFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let words = args.lenght.unwrap_or(50) as usize;

        let truncate_string = args.ellipsis.unwrap_or_else(|| "...".into());

        let l = cmp::max(words, 0);

        let input_string = input.to_kstr();

        let word_list: Vec<&str> = input_string.split(' ').collect();
        let result = if words < word_list.len() {
            let result = itertools::join(word_list.iter().take(l), " ") + truncate_string.as_str();
            Value::scalar(result)
        } else {
            input.clone()
        };
        Ok(result)
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
    fn unit_truncate() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("I often quote ...");
        assert_eq!(unit!(Truncate, input, Value::scalar(17i32)), desired_result);
    }

    #[test]
    fn unit_truncate_negative_length() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("I often quote myself.  It adds spice to my conversation.");
        assert_eq!(
            unit!(Truncate, input, Value::scalar(-17i32)),
            desired_result
        );
    }

    #[test]
    fn unit_truncate_non_string() {
        let input = &Value::scalar(10000000f64);
        let desired_result = tos!("10...");
        assert_eq!(unit!(Truncate, input, Value::scalar(5i32)), desired_result);
    }

    #[test]
    fn unit_truncate_shopify_liquid() {
        // Tests from https://shopify.github.io/liquid/filters/truncate/
        let input = &tos!("Ground control to Major Tom.");

        let desired_result = tos!("Ground control to...");
        assert_eq!(unit!(Truncate, input, Value::scalar(20i32)), desired_result);

        let desired_result = tos!("Ground control, and so on");
        assert_eq!(
            unit!(Truncate, input, Value::scalar(25i32), tos!(", and so on")),
            desired_result
        );

        let desired_result = tos!("Ground control to Ma");
        assert_eq!(
            unit!(Truncate, input, Value::scalar(20i32), tos!("")),
            desired_result
        );
    }

    #[test]
    fn unit_truncate_three_arguments() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        failed!(
            Truncate,
            input,
            Value::scalar(17i32),
            tos!("..."),
            Value::scalar(0i32)
        );
    }

    #[test]
    fn unit_truncate_unicode_codepoints_examples() {
        // The examples below came from the unicode_segmentation documentation.
        //
        // https://unicode-rs.github.io/unicode-segmentation/unicode_segmentation/ ...
        //               ...  trait.UnicodeSegmentation.html#tymethod.graphemes
        //
        // Note that the accents applied to each letter are treated as part of the single grapheme
        // cluster for the applicable letter.
        let input = &tos!("Here is an a\u{310}, e\u{301}, and o\u{308}\u{332}.");
        let desired_result = tos!("Here is an a\u{310}, e\u{301}, ...");
        assert_eq!(unit!(Truncate, input, Value::scalar(20i32)), desired_result);

        // Note that the 🇷🇺🇸🇹 is treated as a single grapheme cluster.
        let input = &tos!("Here is a RUST: 🇷🇺🇸🇹.");
        let desired_result = tos!("Here is a RUST: 🇷🇺...");
        assert_eq!(unit!(Truncate, input, Value::scalar(20i32)), desired_result);
    }

    #[test]
    fn unit_truncate_zero_arguments() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("I often quote myself.  It adds spice to my conv...");
        assert_eq!(unit!(Truncate, input), desired_result);
    }

    #[test]
    fn unit_truncatewords_negative_length() {
        assert_eq!(
            unit!(TruncateWords, tos!("one two three"), Value::scalar(-1_i32)),
            tos!("one two three")
        );
    }

    #[test]
    fn unit_truncatewords_zero_length() {
        assert_eq!(
            unit!(TruncateWords, tos!("one two three"), Value::scalar(0_i32)),
            tos!("...")
        );
    }

    #[test]
    fn unit_truncatewords_no_truncation() {
        assert_eq!(
            unit!(TruncateWords, tos!("one two three"), Value::scalar(4_i32)),
            tos!("one two three")
        );
    }

    #[test]
    fn unit_truncatewords_truncate() {
        assert_eq!(
            unit!(TruncateWords, tos!("one two three"), Value::scalar(2_i32)),
            tos!("one two...")
        );
        assert_eq!(
            unit!(
                TruncateWords,
                tos!("one two three"),
                Value::scalar(2_i32),
                Value::scalar(1_i32)
            ),
            tos!("one two1")
        );
    }

    #[test]
    fn unit_truncatewords_empty_string() {
        assert_eq!(
            unit!(TruncateWords, tos!(""), Value::scalar(1_i32)),
            tos!("")
        );
    }
}
