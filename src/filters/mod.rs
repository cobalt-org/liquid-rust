#[cfg(feature = "extra-filters")]
mod array;
mod date;
mod html;
mod math;
mod url;

#[cfg(feature = "extra-filters")]
pub use self::array::{array_to_sentence_string, pop, push, shift, unshift};
pub use self::date::date;
#[cfg(feature = "extra-filters")]
pub use self::date::date_in_tz;
pub use self::html::{escape, escape_once, newline_to_br, strip_html};
pub use self::math::{abs, at_least, at_most, divided_by, minus, modulo, plus, times};
pub use self::url::{url_decode, url_encode};

use std::borrow::Cow;
use std::cmp;

use itertools;
use liquid_error;
use liquid_value::Scalar;
use liquid_value::Value;
use unicode_segmentation::UnicodeSegmentation;

use interpreter::FilterResult;

pub fn invalid_input<S: Into<Cow<'static, str>>>(cause: S) -> liquid_error::Error {
    liquid_error::Error::with_msg("Invalid input").context("cause", cause)
}

pub fn invalid_argument_count<S: Into<Cow<'static, str>>>(cause: S) -> liquid_error::Error {
    liquid_error::Error::with_msg("Invalid number of arguments").context("cause", cause)
}

pub fn invalid_argument<S: Into<Cow<'static, str>>>(
    position: usize,
    cause: S,
) -> liquid_error::Error {
    liquid_error::Error::with_msg("Invalid argument")
        .context("position", format!("{}", position))
        .context("cause", cause)
}

// Helper functions for the filters.
fn check_args_len(
    args: &[Value],
    required: usize,
    optional: usize,
) -> Result<(), liquid_error::Error> {
    if args.len() < required {
        return Err(invalid_argument_count(format!(
            "expected at least {}, {} given",
            required,
            args.len()
        )));
    }
    if required + optional < args.len() {
        return Err(invalid_argument_count(format!(
            "expected at most {}, {} given",
            required + optional,
            args.len()
        )));
    }
    Ok(())
}

// standardfilters.rb

pub fn size(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    match *input {
        Value::Scalar(ref x) => Ok(Value::scalar(x.to_str().len() as i32)),
        Value::Array(ref x) => Ok(Value::scalar(x.len() as i32)),
        Value::Object(ref x) => Ok(Value::scalar(x.len() as i32)),
        _ => Ok(Value::scalar(0i32)),
    }
}

pub fn downcase(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let s = input.to_str();
    Ok(Value::scalar(s.to_lowercase()))
}

pub fn upcase(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let s = input.to_str();
    Ok(Value::scalar(s.to_uppercase()))
}

pub fn capitalize(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let s = input.to_str().to_owned();
    let mut chars = s.chars();
    let capitalized = match chars.next() {
        Some(first_char) => first_char.to_uppercase().chain(chars).collect(),
        None => String::new(),
    };

    Ok(Value::scalar(capitalized))
}

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

pub fn slice(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 1)?;

    let offset = args[0]
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_argument(0, "Whole number expected"))?;
    let offset = offset as isize;

    let length = args
        .get(1)
        .unwrap_or(&Value::scalar(1))
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_argument(0, "Whole number expected"))?;
    if length < 1 {
        return Err(invalid_argument(1, "Positive number expected"));
    }
    let length = length as isize;

    if let Value::Array(ref input) = *input {
        let (offset, length) = canonicalize_slice(offset, length, input.len());
        Ok(Value::array(
            input
                .iter()
                .skip(offset as usize)
                .take(length as usize)
                .cloned(),
        ))
    } else {
        let input = input.to_str();
        let (offset, length) = canonicalize_slice(offset, length, input.len());
        Ok(Value::scalar(
            input
                .chars()
                .skip(offset as usize)
                .take(length as usize)
                .collect::<String>(),
        ))
    }
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
pub fn truncate(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 2)?;

    let length = args
        .get(0)
        .unwrap_or(&Value::scalar(50i32))
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_argument(0, "Whole number expected"))?;
    let length = length as usize;

    let truncate_string = args
        .get(1)
        .map(Value::to_str)
        .unwrap_or_else(|| "...".into());

    let l = cmp::max(length - truncate_string.len(), 0);

    let input_string = input.to_str();

    let result = if length < input_string.len() {
        let result = UnicodeSegmentation::graphemes(input_string.as_ref(), true)
            .take(l)
            .collect::<Vec<&str>>()
            .join("")
            .to_string()
            + truncate_string.as_ref();
        Value::scalar(result)
    } else {
        input.clone()
    };
    Ok(result)
}

pub fn truncatewords(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 2)?;

    let words = args
        .get(0)
        .unwrap_or(&Value::scalar(50i32))
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_argument(0, "Whole number expected"))?;
    let words = words as usize;

    let truncate_string = args
        .get(1)
        .map(Value::to_str)
        .unwrap_or_else(|| "...".into());

    let l = cmp::max(words, 0);

    let input_string = input.to_str();

    let word_list: Vec<&str> = input_string.split(' ').collect();
    let result = if words < word_list.len() {
        let result = itertools::join(word_list.iter().take(l), " ") + truncate_string.as_ref();
        Value::scalar(result)
    } else {
        input.clone()
    };
    Ok(result)
}

pub fn split(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input.to_str();

    let pattern = args[0].to_str();

    // Split and construct resulting Array
    Ok(Value::Array(
        input
            .split(pattern.as_ref())
            .map(|s| Value::scalar(s.to_owned()))
            .collect(),
    ))
}

/// Removes all whitespace (tabs, spaces, and newlines) from both the left and right side of a
/// string.
///
/// It does not affect spaces between words.  Note that while this works for the case of tabs,
/// spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived Core
/// Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn strip(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let input = input.to_str();
    Ok(Value::scalar(input.trim().to_owned()))
}

/// Removes all whitespaces (tabs, spaces, and newlines) from the beginning of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn lstrip(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let input = input.to_str();
    Ok(Value::scalar(input.trim_left().to_owned()))
}

/// Removes all whitespace (tabs, spaces, and newlines) from the right side of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn rstrip(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let input = input.to_str();
    Ok(Value::scalar(input.trim_right().to_owned()))
}

/// Removes any newline characters (line breaks) from a string.
pub fn strip_newlines(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let input = input.to_str();
    Ok(Value::scalar(
        input
            .chars()
            .filter(|c| *c != '\n' && *c != '\r')
            .collect::<String>(),
    ))
}

pub fn join(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 1)?;

    let glue = args.get(0).map(Value::to_str).unwrap_or_else(|| " ".into());

    let input = input
        .as_array()
        .ok_or_else(|| invalid_input("Array of strings expected"))?;
    let input = input.iter().map(|x| x.to_str());

    Ok(Value::scalar(itertools::join(input, glue.as_ref())))
}

pub fn sort(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    // TODO optional property parameter

    let array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?;
    let mut sorted = array.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
    Ok(Value::array(sorted))
}

pub fn sort_natural(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    // TODO optional property parameter

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

/// Removes any duplicate elements in an array.
///
/// This has an O(n^2) worst-case complexity.
pub fn uniq(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    // TODO optional property parameter

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

/// Reverses the order of the items in an array. `reverse` cannot `reverse` a string.
pub fn reverse(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?;
    let mut reversed = array.clone();
    reversed.reverse();
    Ok(Value::array(reversed))
}

/// Extract `property` from the `Value::Object` elements of an array
pub fn map(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?;

    let property = args[0].to_str();
    let property = Scalar::new(property.into_owned());

    let result: Vec<_> = array
        .iter()
        .filter_map(|v| v.get(&property).cloned())
        .collect();
    Ok(Value::array(result))
}

/// Remove nulls from an iterable.  For hashes, you can specify which property you
/// want to filter out if it maps to Null.
pub fn compact(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 1)?;

    let array = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?;

    // TODO optional property parameter

    let result: Vec<_> = array.iter().filter(|v| !v.is_nil()).cloned().collect();

    Ok(Value::array(result))
}

pub fn replace(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 1)?;

    let input = input.to_str();

    let search = args[0].to_str();
    let replace = args.get(1).map(Value::to_str).unwrap_or_else(|| "".into());

    Ok(Value::scalar(
        input.replace(search.as_ref(), replace.as_ref()),
    ))
}

pub fn replace_first(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 1)?;

    let input = input.to_str();

    let search = args[0].to_str();
    let replace = args.get(1).map(Value::to_str).unwrap_or_else(|| "".into());

    {
        let tokens: Vec<&str> = input.splitn(2, search.as_ref()).collect();
        if tokens.len() == 2 {
            let result = [tokens[0], replace.as_ref(), tokens[1]].join("");
            return Ok(Value::scalar(result));
        }
    }
    Ok(Value::scalar(input.into_owned()))
}

pub fn remove(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input.to_str();

    let string = args[0].to_str();

    Ok(Value::scalar(input.replace(string.as_ref(), "")))
}

pub fn remove_first(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input.to_str();

    let string = args[0].to_str();

    Ok(Value::scalar(
        input.splitn(2, string.as_ref()).collect::<String>(),
    ))
}

pub fn append(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let mut input = input.to_string();

    let string = args[0].to_str();

    input.push_str(string.as_ref());

    Ok(Value::scalar(input))
}

pub fn concat(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input
        .as_array()
        .ok_or_else(|| invalid_input("Array expected"))?;
    let input = input.iter().cloned();

    let array = args[0]
        .as_array()
        .ok_or_else(|| invalid_argument(0, "Array expected"))?;
    let array = array.iter().cloned();

    let result = input.chain(array);
    let result: Vec<_> = result.collect();
    Ok(Value::array(result))
}

pub fn prepend(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let input = input.to_str();

    let mut string = args[0].to_string();

    string.push_str(input.as_ref());

    Ok(Value::scalar(string))
}

pub fn first(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

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

pub fn last(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

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

pub fn round(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 1)?;

    let n = args
        .get(0)
        .unwrap_or(&Value::scalar(0i32))
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_argument(0, "Whole number expected"))?;

    let input = input
        .as_scalar()
        .and_then(Scalar::to_float)
        .ok_or_else(|| invalid_input("Number expected"))?;

    if n == 0 {
        Ok(Value::scalar(input.round() as i32))
    } else if n < 0 {
        Err(invalid_argument(0, "Positive number expected"))
    } else {
        let multiplier = 10.0_f64.powi(n);
        Ok(Value::scalar((input * multiplier).round() / multiplier))
    }
}

pub fn ceil(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let n = input
        .as_scalar()
        .and_then(Scalar::to_float)
        .ok_or_else(|| invalid_input("Number expected"))?;
    Ok(Value::scalar(n.ceil() as i32))
}

pub fn floor(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let n = input
        .as_scalar()
        .and_then(Scalar::to_float)
        .ok_or_else(|| invalid_input("Number expected"))?;
    Ok(Value::scalar(n.floor() as i32))
}

pub fn default(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    if input.is_default() {
        Ok(args[0].clone())
    } else {
        Ok(input.clone())
    }
}

// shopify

#[cfg(feature = "extra-filters")]
pub fn pluralize(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 2, 0)?;

    let n = input
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| invalid_input("Whole number expected"))?;
    if (n as isize) == 1 {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use value::Object;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap_err()
        }};
    }

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_append() {
        assert_eq!(unit!(append, tos!("sam"), &[tos!("son")]), tos!("samson"));
    }

    #[test]
    fn unit_concat_nothing() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[Value::Array(vec![])];
        let result = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        assert_eq!(unit!(concat, input, args), result);
    }

    #[test]
    fn unit_concat_something() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[Value::Array(vec![Value::scalar(3f64), Value::scalar(4f64)])];
        let result = Value::Array(vec![
            Value::scalar(1f64),
            Value::scalar(2f64),
            Value::scalar(3f64),
            Value::scalar(4f64),
        ]);
        assert_eq!(unit!(concat, input, args), result);
    }

    #[test]
    fn unit_concat_mixed() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[Value::Array(vec![Value::scalar(3f64), Value::scalar("a")])];
        let result = Value::Array(vec![
            Value::scalar(1f64),
            Value::scalar(2f64),
            Value::scalar(3f64),
            Value::scalar("a"),
        ]);
        assert_eq!(unit!(concat, input, args), result);
    }

    #[test]
    fn unit_concat_wrong_type() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[Value::scalar(1f64)];
        failed!(concat, input, args);
    }

    #[test]
    fn unit_concat_no_args() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[];
        failed!(concat, input, args);
    }

    #[test]
    fn unit_concat_extra_args() {
        let input = Value::Array(vec![Value::scalar(1f64), Value::scalar(2f64)]);
        let args = &[
            Value::Array(vec![Value::scalar(3f64), Value::scalar("a")]),
            Value::scalar(2f64),
        ];
        failed!(concat, input, args);
    }

    #[test]
    fn unit_capitalize() {
        assert_eq!(unit!(capitalize, tos!("abc")), tos!("Abc"));
        assert_eq!(
            unit!(capitalize, tos!("hello world 21")),
            tos!("Hello world 21")
        );

        // sure that Umlauts work
        assert_eq!(
            unit!(capitalize, tos!("über ètat, y̆es?")),
            tos!("Über ètat, y̆es?")
        );

        // Weird UTF-8 White space is kept – this is a no-break whitespace!
        assert_eq!(
            unit!(capitalize, tos!("hello world​")),
            tos!("Hello world​")
        );

        // The uppercase version of some character are more than one character long
        assert_eq!(unit!(capitalize, tos!("ßß")), tos!("SSß"));
    }

    #[test]
    fn unit_ceil() {
        assert_eq!(unit!(ceil, Value::scalar(1.1f64), &[]), Value::scalar(2f64));
        assert_eq!(unit!(ceil, Value::scalar(1f64), &[]), Value::scalar(1f64));
        assert!(ceil(&Value::scalar(true), &[]).is_err());
    }

    #[test]
    fn unit_downcase() {
        assert_eq!(unit!(downcase, tos!("Abc")), tos!("abc"));
        assert_eq!(
            unit!(downcase, tos!("Hello World 21")),
            tos!("hello world 21")
        );
    }

    #[test]
    fn unit_first() {
        assert_eq!(
            unit!(
                first,
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
            unit!(first, Value::Array(vec![tos!("test"), tos!("two")])),
            tos!("test")
        );
        assert_eq!(unit!(first, Value::Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_floor() {
        assert_eq!(
            unit!(floor, Value::scalar(1.1f64), &[]),
            Value::scalar(1f64)
        );
        assert_eq!(unit!(floor, Value::scalar(1f64), &[]), Value::scalar(1f64));
        assert!(floor(&Value::scalar(true), &[]).is_err());
    }

    #[test]
    fn unit_join() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[tos!(",")];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a,b,c"));
    }

    #[test]
    fn unit_join_bad_input() {
        let input = tos!("a");
        let args = &[tos!(",")];
        let result = join(&input, args);
        assert!(result.is_err());
    }

    #[test]
    fn unit_join_bad_join_string() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[Value::scalar(1f64)];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a1b1c"));
    }

    #[test]
    fn unit_join_no_args() {
        let input = Value::Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a b c"));
    }

    #[test]
    fn unit_join_non_string_element() {
        let input = Value::Array(vec![tos!("a"), Value::scalar(1f64), tos!("c")]);
        let args = &[tos!(",")];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a,1,c"));
    }

    #[test]
    fn unit_sort() {
        let input = &Value::Array(vec![tos!("Z"), tos!("b"), tos!("c"), tos!("a")]);
        let args = &[];
        let desired_result = Value::Array(vec![tos!("Z"), tos!("a"), tos!("b"), tos!("c")]);
        assert_eq!(unit!(sort, input, args), desired_result);
    }

    #[test]
    fn unit_sort_natural() {
        let input = &Value::Array(vec![tos!("Z"), tos!("b"), tos!("c"), tos!("a")]);
        let args = &[];
        let desired_result = Value::Array(vec![tos!("a"), tos!("b"), tos!("c"), tos!("Z")]);
        assert_eq!(unit!(sort_natural, input, args), desired_result);
    }

    #[test]
    fn unit_last() {
        assert_eq!(
            unit!(
                last,
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
            unit!(last, Value::Array(vec![tos!("test"), tos!("last")])),
            tos!("last")
        );
        assert_eq!(unit!(last, Value::Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_lstrip() {
        let input = &tos!(" 	 \n \r test");
        let args = &[];
        let desired_result = tos!("test");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_non_string() {
        let input = &Value::scalar(0f64);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        let args = &[Value::scalar(0f64)];
        failed!(lstrip, input, args);
    }

    #[test]
    fn unit_lstrip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/lstrip/
        let input = &tos!("          So much room for activities!          ");
        let args = &[];
        let desired_result = tos!("So much room for activities!          ");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_trailing_sequence() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let args = &[];
        let desired_result = tos!("test 	 \n \r ");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_trailing_sequence_only() {
        let input = &tos!("test 	 \n \r ");
        let args = &[];
        let desired_result = tos!("test 	 \n \r ");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_pluralize() {
        assert_eq!(
            unit!(pluralize, Value::scalar(1i32), &[tos!("one"), tos!("many")]),
            tos!("one")
        );

        assert_eq!(
            unit!(pluralize, Value::scalar(2i32), &[tos!("one"), tos!("many")]),
            tos!("many")
        );
    }

    #[test]
    fn unit_prepend() {
        assert_eq!(
            unit!(prepend, tos!("barbar"), &[tos!("foo")]),
            tos!("foobarbar")
        );
    }

    #[test]
    fn unit_remove() {
        assert_eq!(unit!(remove, tos!("barbar"), &[tos!("bar")]), tos!(""));
        assert_eq!(unit!(remove, tos!("barbar"), &[tos!("")]), tos!("barbar"));
        assert_eq!(unit!(remove, tos!("barbar"), &[tos!("barbar")]), tos!(""));
        assert_eq!(unit!(remove, tos!("barbar"), &[tos!("a")]), tos!("brbr"));
    }

    #[test]
    fn unit_remove_first() {
        assert_eq!(
            unit!(remove_first, tos!("barbar"), &[tos!("bar")]),
            tos!("bar")
        );
        assert_eq!(
            unit!(remove_first, tos!("barbar"), &[tos!("")]),
            tos!("barbar")
        );
        assert_eq!(
            unit!(remove_first, tos!("barbar"), &[tos!("barbar")]),
            tos!("")
        );
        assert_eq!(
            unit!(remove_first, tos!("barbar"), &[tos!("a")]),
            tos!("brbar")
        );
    }

    #[test]
    fn unit_replace() {
        assert_eq!(
            unit!(replace, tos!("barbar"), &[tos!("bar"), tos!("foo")]),
            tos!("foofoo")
        );
    }

    #[test]
    fn unit_replace_first() {
        assert_eq!(
            unit!(replace_first, tos!("barbar"), &[tos!("bar"), tos!("foo")]),
            tos!("foobar")
        );
        assert_eq!(
            unit!(replace_first, tos!("barxoxo"), &[tos!("xo"), tos!("foo")]),
            tos!("barfooxo")
        );
        assert_eq!(
            unit!(replace_first, tos!(""), &[tos!("bar"), tos!("foo")]),
            tos!("")
        );
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
        let args = &[];
        let desired_result = Value::Array(vec![
            tos!("plums"),
            tos!("peaches"),
            tos!("oranges"),
            tos!("apples"),
        ]);
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_array() {
        let input = &Value::Array(vec![
            Value::scalar(3f64),
            Value::scalar(1f64),
            Value::scalar(2f64),
        ]);
        let args = &[];
        let desired_result = Value::Array(vec![
            Value::scalar(2f64),
            Value::scalar(1f64),
            Value::scalar(3f64),
        ]);
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_array_extra_args() {
        let input = &Value::Array(vec![
            Value::scalar(3f64),
            Value::scalar(1f64),
            Value::scalar(2f64),
        ]);
        let args = &[Value::scalar(0f64)];
        failed!(reverse, input, args);
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
        let args = &[];
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
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_string() {
        let input = &tos!("abc");
        let args = &[];
        failed!(reverse, input, args);
    }

    #[test]
    fn unit_rstrip() {
        let input = &tos!("test 	 \n \r ");
        let args = &[];
        let desired_result = tos!("test");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_rstrip_leading_sequence() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let args = &[];
        let desired_result = tos!(" 	 \n \r test");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_rstrip_leading_sequence_only() {
        let input = &tos!(" 	 \n \r test");
        let args = &[];
        let desired_result = tos!(" 	 \n \r test");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_rstrip_non_string() {
        let input = &Value::scalar(0f64);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_rstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        let args = &[Value::scalar(0f64)];
        failed!(rstrip, input, args);
    }

    #[test]
    fn unit_rstrip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/rstrip/
        let input = &tos!("          So much room for activities!          ");
        let args = &[];
        let desired_result = tos!("          So much room for activities!");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_round() {
        assert_eq!(
            unit!(round, Value::scalar(1.1f64), &[]),
            Value::scalar(1i32)
        );
        assert_eq!(
            unit!(round, Value::scalar(1.5f64), &[]),
            Value::scalar(2i32)
        );
        assert_eq!(unit!(round, Value::scalar(2f64), &[]), Value::scalar(2i32));
        assert!(round(&Value::scalar(true), &[]).is_err());
    }

    #[test]
    fn unit_round_precision() {
        assert_eq!(
            unit!(round, Value::scalar(1.1f64), &[Value::scalar(0i32)]),
            Value::scalar(1f64)
        );
        assert_eq!(
            unit!(round, Value::scalar(1.5f64), &[Value::scalar(1i32)]),
            Value::scalar(1.5f64)
        );
        assert_eq!(
            unit!(round, Value::scalar(3.14159f64), &[Value::scalar(3i32)]),
            Value::scalar(3.142f64)
        );
    }

    #[test]
    fn unit_size() {
        assert_eq!(unit!(size, tos!("abc")), Value::scalar(3f64));
        assert_eq!(
            unit!(size, tos!("this has 22 characters")),
            Value::scalar(22f64)
        );
        assert_eq!(
            unit!(
                size,
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
    fn unit_split() {
        assert_eq!(
            unit!(split, tos!("a, b, c"), &[tos!(", ")]),
            Value::Array(vec![tos!("a"), tos!("b"), tos!("c")])
        );
        assert_eq!(
            unit!(split, tos!("a~b"), &[tos!("~")]),
            Value::Array(vec![tos!("a"), tos!("b")])
        );
    }

    #[test]
    fn unit_split_bad_split_string() {
        let input = tos!("a,b,c");
        let args = &[Value::scalar(1f64)];
        let desired_result = Value::Array(vec![tos!("a,b,c")]);
        assert_eq!(unit!(split, input, args), desired_result);
    }

    #[test]
    fn unit_split_no_args() {
        let input = tos!("a,b,c");
        let args = &[];
        let result = split(&input, args);
        assert!(result.is_err());
    }

    #[test]
    fn unit_strip() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let args = &[];
        let desired_result = tos!("test");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_leading_sequence_only() {
        let input = &tos!(" 	 \n \r test");
        let args = &[];
        let desired_result = tos!("test");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_non_string() {
        let input = &Value::scalar(0f64);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_one_argument() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let args = &[Value::scalar(0f64)];
        failed!(strip, input, args);
    }

    #[test]
    fn unit_strip_shopify_liquid() {
        // One test from https://shopify.github.io/liquid/filters/strip/
        let input = &tos!("          So much room for activities!          ");
        let args = &[];
        let desired_result = tos!("So much room for activities!");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_trailing_sequence_only() {
        let input = &tos!("test 	 \n \r ");
        let args = &[];
        let desired_result = tos!("test");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines() {
        let input = &tos!("a\nb\n");
        let args = &[];
        let desired_result = tos!("ab");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_between_only() {
        let input = &tos!("a\nb");
        let args = &[];
        let desired_result = tos!("ab");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_leading_only() {
        let input = &tos!("\nab");
        let args = &[];
        let desired_result = tos!("ab");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_non_string() {
        let input = &Value::scalar(0f64);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_one_argument() {
        let input = &tos!("ab\n");
        let args = &[Value::scalar(0f64)];
        failed!(strip_newlines, input, args);
    }

    #[test]
    fn unit_strip_newlines_shopify_liquid() {
        // Test from https://shopify.github.io/liquid/filters/strip_newlines/
        let input = &tos!("\nHello\nthere\n");
        let args = &[];
        let desired_result = tos!("Hellothere");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_trailing_only() {
        let input = &tos!("ab\n");
        let args = &[];
        let desired_result = tos!("ab");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_truncate() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Value::scalar(17i32)];
        let desired_result = tos!("I often quote ...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_negative_length() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Value::scalar(-17i32)];
        let desired_result = tos!("I often quote myself.  It adds spice to my conversation.");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_non_string() {
        let input = &Value::scalar(10000000f64);
        let args = &[Value::scalar(5i32)];
        let desired_result = tos!("10...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_shopify_liquid() {
        // Tests from https://shopify.github.io/liquid/filters/truncate/
        let input = &tos!("Ground control to Major Tom.");
        let args = &[Value::scalar(20i32)];
        let desired_result = tos!("Ground control to...");
        assert_eq!(unit!(truncate, input, args), desired_result);

        let args = &[Value::scalar(25i32), tos!(", and so on")];
        let desired_result = tos!("Ground control, and so on");
        assert_eq!(unit!(truncate, input, args), desired_result);

        let args = &[Value::scalar(20i32), tos!("")];
        let desired_result = tos!("Ground control to Ma");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_three_arguments() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Value::scalar(17i32), tos!("..."), Value::scalar(0i32)];
        failed!(truncate, input, args);
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
        let args = &[Value::scalar(20i32)];
        let desired_result = tos!("Here is an a\u{310}, e\u{301}, ...");
        assert_eq!(unit!(truncate, input, args), desired_result);

        // Note that the 🇷🇺🇸🇹 is treated as a single grapheme cluster.
        let input = &tos!("Here is a RUST: 🇷🇺🇸🇹.");
        let args = &[Value::scalar(20i32)];
        let desired_result = tos!("Here is a RUST: 🇷🇺...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_zero_arguments() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[];
        let desired_result = tos!("I often quote myself.  It adds spice to my conv...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncatewords_negative_length() {
        assert_eq!(
            unit!(
                truncatewords,
                tos!("one two three"),
                &[Value::scalar(-1_i32)]
            ),
            tos!("one two three")
        );
    }

    #[test]
    fn unit_truncatewords_zero_length() {
        assert_eq!(
            unit!(
                truncatewords,
                tos!("one two three"),
                &[Value::scalar(0_i32)]
            ),
            tos!("...")
        );
    }

    #[test]
    fn unit_truncatewords_no_truncation() {
        assert_eq!(
            unit!(
                truncatewords,
                tos!("one two three"),
                &[Value::scalar(4_i32)]
            ),
            tos!("one two three")
        );
    }

    #[test]
    fn unit_truncatewords_truncate() {
        assert_eq!(
            unit!(
                truncatewords,
                tos!("one two three"),
                &[Value::scalar(2_i32)]
            ),
            tos!("one two...")
        );
        assert_eq!(
            unit!(
                truncatewords,
                tos!("one two three"),
                &[Value::scalar(2_i32), Value::scalar(1_i32)]
            ),
            tos!("one two1")
        );
    }

    #[test]
    fn unit_truncatewords_empty_string() {
        assert_eq!(
            unit!(truncatewords, tos!(""), &[Value::scalar(1_i32)]),
            tos!("")
        );
    }

    #[test]
    fn unit_uniq() {
        let input = &Value::Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        let args = &[];
        let desired_result = Value::Array(vec![tos!("a"), tos!("b")]);
        assert_eq!(unit!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_uniq_non_array() {
        let input = &Value::scalar(0f64);
        let args = &[];
        failed!(uniq, input, args);
    }

    #[test]
    fn unit_uniq_one_argument() {
        let input = &Value::Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        let args = &[Value::scalar(0f64)];
        failed!(uniq, input, args);
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
        let args = &[];
        let desired_result = Value::Array(vec![tos!("ants"), tos!("bugs"), tos!("bees")]);
        assert_eq!(unit!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(upcase, tos!("abc")), tos!("ABC"));
        assert_eq!(
            unit!(upcase, tos!("Hello World 21")),
            tos!("HELLO WORLD 21")
        );
    }

    #[test]
    fn unit_default() {
        assert_eq!(unit!(default, tos!(""), &[tos!("bar")]), tos!("bar"));
        assert_eq!(unit!(default, tos!("foo"), &[tos!("bar")]), tos!("foo"));
        assert_eq!(
            unit!(default, Value::scalar(0_f64), &[tos!("bar")]),
            Value::scalar(0_f64)
        );
        assert_eq!(
            unit!(default, Value::Array(vec![]), &[Value::scalar(1_f64)]),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(
                default,
                Value::Array(vec![tos!("")]),
                &[Value::scalar(1_f64)]
            ),
            Value::Array(vec![tos!("")])
        );
        assert_eq!(
            unit!(
                default,
                Value::Object(Object::new()),
                &[Value::scalar(1_f64)]
            ),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(default, Value::scalar(false), &[Value::scalar(1_f64)]),
            Value::scalar(1_f64)
        );
        assert_eq!(
            unit!(default, Value::scalar(true), &[Value::scalar(1_f64)]),
            Value::scalar(true)
        );
    }
}
