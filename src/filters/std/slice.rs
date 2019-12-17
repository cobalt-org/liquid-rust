use filters::invalid_argument;
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::Value;
use std::cmp;

fn canonicalize_slice(
    slice_offset: isize,
    slice_length: isize,
    vec_length: usize,
) -> (usize, usize) {
    let vec_length = vec_length as isize;

    // Cap slice_offset
    let slice_offset = cmp::min(slice_offset, vec_length);
    // Reverse indexing
    let slice_offset = if slice_offset < 0 {
        slice_offset + vec_length
    } else {
        slice_offset
    };

    // Cap slice_length
    let slice_length = if slice_offset + slice_length > vec_length {
        vec_length - slice_offset
    } else {
        slice_length
    };

    (slice_offset as usize, slice_length as usize)
}

#[derive(Debug, FilterParameters)]
struct SliceArgs {
    #[parameter(description = "The offset of the slice.", arg_type = "integer")]
    offset: Expression,

    #[parameter(description = "The length of the slice.", arg_type = "integer")]
    length: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "slice",
    description = "Takes a slice of a given string or array.",
    parameters(SliceArgs),
    parsed(SliceFilter)
)]
pub struct Slice;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "slice"]
struct SliceFilter {
    #[parameters]
    args: SliceArgs,
}

impl Filter for SliceFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let offset = args.offset as isize;
        let length = args.length.unwrap_or(1) as isize;

        if length < 1 {
            return invalid_argument("length", "Positive number expected").into_err();
        }

        if let Value::Array(input) = input {
            let (offset, length) = canonicalize_slice(offset, length, input.len());
            Ok(Value::array(
                input.iter().skip(offset).take(length).cloned(),
            ))
        } else {
            let input = input.to_sstr();
            let (offset, length) = canonicalize_slice(offset, length, input.len());
            Ok(Value::scalar(
                input.chars().skip(offset).take(length).collect::<String>(),
            ))
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
    fn unit_slice() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("ot");
        assert_eq!(unit!(Slice, input, tos!(10), tos!(2)), desired_result);
    }

    #[test]
    fn unit_slice_no_lenght_specified() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("t");
        assert_eq!(unit!(Slice, input, tos!(4)), desired_result);
    }

    #[test]
    fn unit_slice_negative_offset() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let desired_result = tos!("ver");
        assert_eq!(unit!(Slice, input, tos!(-10), tos!(3)), desired_result);
    }

    #[test]
    fn unit_slice_non_positive_lenght() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");

        failed!(Slice, input, tos!(-10), tos!(0));
        failed!(Slice, input, tos!(-10), tos!(-1));
    }
}
