use filters::{invalid_argument, invalid_input};
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::{Scalar, Value};
use std::cmp;

#[derive(Debug, FilterParameters)]
struct JoinArgs {
    #[parameter(
        description = "The separator between each element in the string.",
        arg_type = "str"
    )]
    separator: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "join",
    description = "Combines the items in an array into a single string using the argument as a separator.",
    parameters(JoinArgs),
    parsed(JoinFilter)
)]
pub struct Join;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "join"]
struct JoinFilter {
    #[parameters]
    args: JoinArgs,
}

impl Filter for JoinFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let separator = args.separator.unwrap_or_else(|| " ".into());

        let input = input
            .as_array()
            .ok_or_else(|| invalid_input("Array of strings expected"))?;
        let input = input.iter().map(|x| x.to_str());

        Ok(Value::scalar(itertools::join(input, separator.as_ref())))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "sort",
    description = "Sorts items in an array. The order of the sorted array is case-sensitive.",
    parsed(SortFilter)
)]
pub struct Sort;

#[derive(Debug, Default, Display_filter)]
#[name = "sort"]
struct SortFilter;

impl Filter for SortFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        // TODO(#333) optional property parameter

        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        let mut sorted = array.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
        Ok(Value::array(sorted))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "sort_natural",
    description = "Sorts items in an array.",
    parsed(SortNaturalFilter)
)]
pub struct SortNatural;

#[derive(Debug, Default, Display_filter)]
#[name = "sort_natural"]
struct SortNaturalFilter;

impl Filter for SortNaturalFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        // TODO(#334) optional property parameter

        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        let mut sorted: Vec<_> = array
            .iter()
            .map(|v| (v.to_str().to_lowercase(), v.clone()))
            .collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
        let result: Vec<_> = sorted.into_iter().map(|(_, v)| v).collect();
        Ok(Value::array(result))
    }
}

/// Removes any duplicate elements in an array.
///
/// This has an O(n^2) worst-case complexity.
#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "uniq",
    description = "Removes any duplicate elements in an array.",
    parsed(UniqFilter)
)]
pub struct Uniq;

#[derive(Debug, Default, Display_filter)]
#[name = "uniq"]
struct UniqFilter;

impl Filter for UniqFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        // TODO(#267) optional property parameter

        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        let mut deduped: Vec<Value> = Vec::new();
        for x in array.iter() {
            if !deduped.contains(x) {
                deduped.push(x.clone())
            }
        }
        Ok(Value::array(deduped))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "reverse",
    description = "Reverses the order of the items in an array.",
    parsed(ReverseFilter)
)]
pub struct Reverse;

#[derive(Debug, Default, Display_filter)]
#[name = "reverse"]
struct ReverseFilter;

impl Filter for ReverseFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        let mut reversed = array.clone();
        reversed.reverse();
        Ok(Value::array(reversed))
    }
}

#[derive(Debug, FilterParameters)]
struct MapArgs {
    #[parameter(
        description = "The property to be extracted from the values in the input.",
        arg_type = "str"
    )]
    property: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "map",
    description = "Extract `property` from the `Value::Object` elements of an array.",
    parameters(MapArgs),
    parsed(MapFilter)
)]
pub struct Map;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "map"]
struct MapFilter {
    #[parameters]
    args: MapArgs,
}

impl Filter for MapFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;

        let property = Scalar::new(args.property.into_owned());

        let result: Vec<_> = array
            .iter()
            .filter_map(|v| v.get(&property).cloned())
            .collect();
        Ok(Value::array(result))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "compact",
    description = "Remove nulls from an iterable.",
    parsed(CompactFilter)
)]
pub struct Compact;

#[derive(Debug, Default, Display_filter)]
#[name = "compact"]
struct CompactFilter;

impl Filter for CompactFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;

        // TODO(#335) optional property parameter

        let result: Vec<_> = array.iter().filter(|v| !v.is_nil()).cloned().collect();

        Ok(Value::array(result))
    }
}

#[derive(Debug, FilterParameters)]
struct ConcatArgs {
    #[parameter(description = "The array to concatenate the input with.")]
    array: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "concat",
    description = "Concatenates the input array with a given array.",
    parameters(ConcatArgs),
    parsed(ConcatFilter)
)]
pub struct Concat;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "concat"]
struct ConcatFilter {
    #[parameters]
    args: ConcatArgs,
}

impl Filter for ConcatFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let input = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        let input = input.iter().cloned();

        let array = args
            .array
            .as_array()
            .ok_or_else(|| invalid_argument("array", "Array expected"))?;
        let array = array.iter().cloned();

        let result = input.chain(array);
        let result: Vec<_> = result.collect();
        Ok(Value::array(result))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "first",
    description = "Returns the first item of an array.",
    parsed(FirstFilter)
)]
pub struct First;

#[derive(Debug, Default, Display_filter)]
#[name = "first"]
struct FirstFilter;

impl Filter for FirstFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        match *input {
            Value::Scalar(ref x) => {
                let c = x
                    .to_str()
                    .chars()
                    .next()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "".to_owned());
                Ok(Value::scalar(c))
            }
            Value::Array(ref x) => Ok(x.first().cloned().unwrap_or_else(|| Value::scalar(""))),
            _ => Err(invalid_input("String or Array expected")),
        }
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "last",
    description = "Returns the last item of an array.",
    parsed(LastFilter)
)]
pub struct Last;

#[derive(Debug, Default, Display_filter)]
#[name = "last"]
struct LastFilter;

impl Filter for LastFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        match *input {
            Value::Scalar(ref x) => {
                let c = x
                    .to_str()
                    .chars()
                    .last()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "".to_owned());
                Ok(Value::scalar(c))
            }
            Value::Array(ref x) => Ok(x.last().cloned().unwrap_or_else(|| Value::scalar(""))),
            _ => Err(invalid_input("String or Array expected")),
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
    fn unit_concat_nothing() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let result = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        assert_eq!(unit!(Concat, input, Value::Array(vec![])), result);
    }

    #[test]
    fn unit_concat_something() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let result = Value::Array(vec![
            Value::scalar(1f64),
            Value::scalar(2f64),
            Value::scalar(3f64),
            Value::scalar(4f64),
        ]);
        assert_eq!(
            unit!(
                Concat,
                input,
                Value::Array(vec![Value::scalar(3f64), Value::scalar(4f64)])
            ),
            result
        );
    }

    #[test]
    fn unit_concat_mixed() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let result = Value::Array(vec![
            Value::scalar(1f64),
            Value::scalar(2f64),
            Value::scalar(3f64),
            Value::scalar("a"),
        ]);
        assert_eq!(
            unit!(
                Concat,
                input,
                Value::Array(vec![Value::scalar(3f64), Value::scalar("a")])
            ),
            result
        );
    }

    #[test]
    fn unit_concat_wrong_type() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        failed!(Concat, input, Value::scalar(1f64));
    }

    #[test]
    fn unit_concat_no_args() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        failed!(Concat, input);
    }

    #[test]
    fn unit_concat_extra_args() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        failed!(
            Concat,
            input,
            Value::Array(vec![Value::scalar(3f64), Value::scalar("a")]),
            Value::scalar(2f64)
        );
    }

    #[test]
    fn unit_first() {
        assert_eq!(
            unit!(
                First,
                Value::Array(vec![
                    Value::scalar(0f64),
                    Value::scalar(1f64),
                    Value::scalar(2f64),
                    Value::scalar(3f64),
                    Value::scalar(4f64),
                ])
            ),
            Value::scalar(0f64)
        );
        assert_eq!(
            unit!(First, Value::Array(vec![tos!("test"), tos!("two")])),
            tos!("test")
        );
        assert_eq!(unit!(First, Value::Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_join() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        assert_eq!(unit!(Join, input, tos!(",")), tos!("a,b,c"));
    }

    #[test]
    fn unit_join_bad_input() {
        let input = tos!("a");
        failed!(Join, input, tos!(","));
    }

    #[test]
    fn unit_join_bad_join_string() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        assert_eq!(unit!(Join, input, Value::scalar(1f64)), tos!("a1b1c"));
    }

    #[test]
    fn unit_join_no_args() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        assert_eq!(unit!(Join, input), tos!("a b c"));
    }

    #[test]
    fn unit_join_non_string_element() {
        let input = Value::Array(vec![tos!("a"), Value::scalar(1f64), tos!("c")]);
        assert_eq!(unit!(Join, input, tos!(",")), tos!("a,1,c"));
    }

    #[test]
    fn unit_sort() {
        let input = &Value::Array(vec![tos!("Z"), tos!("b"), tos!("c"), tos!("a")]);
        let desired_result = Value::Array(vec![tos!("Z"), tos!("a"), tos!("b"), tos!("c")]);
        assert_eq!(unit!(Sort, input), desired_result);
    }

    #[test]
    fn unit_sort_natural() {
        let input = &Value::Array(vec![tos!("Z"), tos!("b"), tos!("c"), tos!("a")]);
        let desired_result = Value::Array(vec![tos!("a"), tos!("b"), tos!("c"), tos!("Z")]);
        assert_eq!(unit!(SortNatural, input), desired_result);
    }

    #[test]
    fn unit_last() {
        assert_eq!(
            unit!(
                Last,
                Value::Array(vec![
                    Value::scalar(0f64),
                    Value::scalar(1f64),
                    Value::scalar(2f64),
                    Value::scalar(3f64),
                    Value::scalar(4f64),
                ])
            ),
            Value::scalar(4f64)
        );
        assert_eq!(
            unit!(Last, Value::Array(vec![tos!("test"), tos!("last")])),
            tos!("last")
        );
        assert_eq!(unit!(Last, Value::Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_reverse_apples_oranges_peaches_plums() {
        // First example from https://shopify.github.io/liquid/filters/reverse/
        let input = &Value::Array(vec![
            tos!("apples"),
            tos!("oranges"),
            tos!("peaches"),
            tos!("plums"),
        ]);
        let desired_result = Value::Array(vec![
            tos!("plums"),
            tos!("peaches"),
            tos!("oranges"),
            tos!("apples"),
        ]);
        assert_eq!(unit!(Reverse, input), desired_result);
    }

    #[test]
    fn unit_reverse_array() {
        let input = &Value::Array(vec![
            Value::scalar(3f64),
            Value::scalar(1f64),
            Value::scalar(2f64),
        ]);
        let desired_result = Value::Array(vec![
            Value::scalar(2f64),
            Value::scalar(1f64),
            Value::scalar(3f64),
        ]);
        assert_eq!(unit!(Reverse, input), desired_result);
    }

    #[test]
    fn unit_reverse_array_extra_args() {
        let input = &Value::Array(vec![
            Value::scalar(3f64),
            Value::scalar(1f64),
            Value::scalar(2f64),
        ]);
        failed!(Reverse, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_reverse_ground_control_major_tom() {
        // Second example from https://shopify.github.io/liquid/filters/reverse/
        let input = &Value::Array(vec![
            tos!("G"),
            tos!("r"),
            tos!("o"),
            tos!("u"),
            tos!("n"),
            tos!("d"),
            tos!(" "),
            tos!("c"),
            tos!("o"),
            tos!("n"),
            tos!("t"),
            tos!("r"),
            tos!("o"),
            tos!("l"),
            tos!(" "),
            tos!("t"),
            tos!("o"),
            tos!(" "),
            tos!("M"),
            tos!("a"),
            tos!("j"),
            tos!("o"),
            tos!("r"),
            tos!(" "),
            tos!("T"),
            tos!("o"),
            tos!("m"),
            tos!("."),
        ]);
        let desired_result = Value::Array(vec![
            tos!("."),
            tos!("m"),
            tos!("o"),
            tos!("T"),
            tos!(" "),
            tos!("r"),
            tos!("o"),
            tos!("j"),
            tos!("a"),
            tos!("M"),
            tos!(" "),
            tos!("o"),
            tos!("t"),
            tos!(" "),
            tos!("l"),
            tos!("o"),
            tos!("r"),
            tos!("t"),
            tos!("n"),
            tos!("o"),
            tos!("c"),
            tos!(" "),
            tos!("d"),
            tos!("n"),
            tos!("u"),
            tos!("o"),
            tos!("r"),
            tos!("G"),
        ]);
        assert_eq!(unit!(Reverse, input), desired_result);
    }

    #[test]
    fn unit_reverse_string() {
        let input = &tos!("abc");
        failed!(Reverse, input);
    }

    #[test]
    fn unit_uniq() {
        let input = &Value::Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        let desired_result = Value::Array(vec![tos!("a"), tos!("b")]);
        assert_eq!(unit!(Uniq, input), desired_result);
    }

    #[test]
    fn unit_uniq_non_array() {
        let input = &Value::scalar(0f64);
        failed!(Uniq, input);
    }

    #[test]
    fn unit_uniq_one_argument() {
        let input = &Value::Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        failed!(Uniq, input, Value::scalar(0f64));
    }

    #[test]
    fn unit_uniq_shopify_liquid() {
        // Test from https://shopify.github.io/liquid/filters/uniq/
        let input = &Value::Array(vec![
            tos!("ants"),
            tos!("bugs"),
            tos!("bees"),
            tos!("bugs"),
            tos!("ants"),
        ]);
        let desired_result = Value::Array(vec![tos!("ants"), tos!("bugs"), tos!("bees")]);
        assert_eq!(unit!(Uniq, input), desired_result);
    }
}
