use std::fmt;
use std::error::Error;
use std::cmp;

use value::Value;
use value::Value::{Array, Bool, Num, Object, Str};

use chrono::DateTime;

#[cfg(feature = "extra-filters")]
use chrono::FixedOffset;

use self::FilterError::*;

use regex::Regex;
use itertools;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Eq)]
pub enum FilterError {
    InvalidType(String),
    InvalidArgumentCount(String),
    InvalidArgument(u16, String), // (position, "expected / given ")
}

impl FilterError {
    pub fn invalid_type<T>(s: &str) -> Result<T, FilterError> {
        Err(FilterError::InvalidType(s.to_owned()))
    }
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidType(ref e) => write!(f, "Invalid type : {}", e),
            InvalidArgumentCount(ref e) => write!(f, "Invalid number of arguments : {}", e),
            InvalidArgument(ref pos, ref e) => {
                write!(f, "Invalid argument given at position {} : {}", pos, e)
            }
        }
    }
}

impl Error for FilterError {
    fn description(&self) -> &str {
        match *self {
            InvalidType(ref e) |
            InvalidArgumentCount(ref e) |
            InvalidArgument(_, ref e) => e,
        }
    }
}

pub type FilterResult = Result<Value, FilterError>;
pub type Filter = Fn(&Value, &[Value]) -> FilterResult;

// Helper functions for the filters.
fn check_args_len(args: &[Value], expected_len: usize) -> Result<(), FilterError> {
    if args.len() != expected_len {
        return Err(InvalidArgumentCount(format!("expected {}, {} given",
                                                expected_len,
                                                args.len())));
    }
    Ok(())
}

/// Returns the number of already escaped characters.
fn nr_escaped(text: &str) -> usize {
    for prefix in &["lt;", "gt;", "#39;", "quot;", "amp;"] {
        if text.starts_with(prefix) {
            return prefix.len();
        }
    }
    0
}

// The code is adapted from
// https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/escape.rs
// Retrieved 2016-11-19.
fn _escape(input: &Value, args: &[Value], once_p: bool) -> FilterResult {
    try!(check_args_len(args, 0));

    let s = input.as_str().ok_or_else(|| InvalidType("String expected".to_owned()))?;
    let mut result = String::new();
    let mut last = 0;
    let mut skip = 0;
    for (i, c) in s.chars().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c as char {
            '<' | '>' | '\'' | '"' | '&' => {
                result.push_str(&s[last..i]);
                last = i + 1;
                let escaped = match c as char {
                    '<' => "&lt;",
                    '>' => "&gt;",
                    '\'' => "&#39;",
                    '"' => "&quot;",
                    '&' => {
                        if once_p {
                            skip = nr_escaped(&s[last..]);
                        }
                        if skip == 0 { "&amp;" } else { "&" }
                    }
                    _ => unreachable!(),
                };
                result.push_str(escaped);
            }
            _ => {}
        }
    }
    if last < s.len() {
        result.push_str(&s[last..]);
    }
    Ok(Str(result))
}

// standardfilters.rb

pub fn size(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => Ok(Num(x.len() as f32)),
        Array(ref x) => Ok(Num(x.len() as f32)),
        Object(ref x) => Ok(Num(x.len() as f32)),
        _ => Ok(Num(0f32)),
    }
}

pub fn downcase(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let s = input.to_string();
    Ok(Str(s.to_lowercase()))
}

pub fn upcase(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let s = input.to_string();
    Ok(Str(s.to_uppercase()))
}

pub fn capitalize(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let s = input.to_string();
    let mut chars = s.chars();
    let capitalized = match chars.next() {
        Some(first_char) => first_char.to_uppercase().chain(chars).collect(),
        None => String::new(),
    };

    Ok(Str(capitalized))
}

pub fn escape(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, false)
}

pub fn escape_once(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, true)
}

// TODO url_encode

// TODO url_decode

fn canonicalize_slice(slice_offset: isize,
                      slice_length: isize,
                      vec_length: usize)
                      -> (usize, usize) {
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
    if args.len() < 1 || args.len() > 2 {
        return Err(InvalidArgumentCount(format!("expected one or two arguments, {} given",
                                                args.len())));
    }

    let offset = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;
    let offset = offset as isize;

    let length = match args.get(1) {
        Some(&Num(x)) if x > 0f32 => x as isize,
        Some(_) => return Err(InvalidArgument(1, "Positive number expected".to_owned())),
        None => 1,
    };

    if let Value::Array(ref input) = *input {
        let (offset, length) = canonicalize_slice(offset, length, input.len());
        Ok(Value::Array(input.iter()
            .skip(offset as usize)
            .take(length as usize)
            .cloned()
            .collect()))
    } else {
        let input = input.to_string();
        let (offset, length) = canonicalize_slice(offset, length, input.len());
        Ok(Value::Str(input.chars().skip(offset as usize).take(length as usize).collect()))
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
    if 2 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one or two arguments, {} given",
                                                args.len())));
    }

    let length = args.get(0)
        .unwrap_or(&Value::Num(50f32))
        .as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;
    let length = length as usize;

    let truncate_string = args.get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "...".to_owned());

    let l = cmp::max(length - truncate_string.len(), 0);

    let input = input.to_string();

    let result = if length < input.len() {
        UnicodeSegmentation::graphemes(input.as_str(), true)
            .take(l)
            .collect::<Vec<&str>>()
            .join("")
            .to_string() + truncate_string.as_str()
    } else {
        input
    };
    Ok(Str(result))
}

pub fn truncatewords(input: &Value, args: &[Value]) -> FilterResult {
    if 2 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one or two arguments, {} given",
                                                args.len())));
    }

    let words = args.get(0)
        .unwrap_or(&Value::Num(15f32))
        .as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;
    let words = words as usize;

    let truncate_string = args.get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "...".to_owned());

    let l = cmp::max(words, 0);

    let input_string = input.to_string();
    let word_list: Vec<&str> = input_string.split(' ').collect();
    let result = if words < word_list.len() {
        let result = itertools::join(word_list.iter().take(l), " ") + truncate_string.as_str();
        Str(result)
    } else {
        input.clone()
    };
    Ok(result)
}

pub fn split(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.to_string();

    let pattern = args[0].to_string();

    // Split and construct resulting Array
    Ok(Array(input.split(pattern.as_str())
        .map(|x| Str(x.to_owned()))
        .collect()))
}

/// Removes all whitespace (tabs, spaces, and newlines) from both the left and right side of a
/// string.
///
/// It does not affect spaces between words.  Note that while this works for the case of tabs,
/// spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived Core
/// Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn strip(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let input = input.to_string();
    Ok(Str(input.trim().to_string()))
}

/// Removes all whitespaces (tabs, spaces, and newlines) from the beginning of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn lstrip(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let input = input.to_string();
    Ok(Str(input.trim_left().to_string()))
}

/// Removes all whitespace (tabs, spaces, and newlines) from the right side of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn rstrip(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let input = input.to_string();
    Ok(Str(input.trim_right().to_string()))
}

pub fn strip_html(input: &Value, args: &[Value]) -> FilterResult {
    lazy_static! {
        // regexps taken from https://git.io/vXbgS
        static ref MATCHERS: [Regex; 4] = [Regex::new(r"(?is)<script.*?</script>").unwrap(),
                                           Regex::new(r"(?is)<style.*?</style>").unwrap(),
                                           Regex::new(r"(?is)<!--.*?-->").unwrap(),
                                           Regex::new(r"(?is)<.*?>").unwrap()];
    }
    try!(check_args_len(args, 0));

    let input = input.to_string();

    let result = MATCHERS.iter()
        .fold(input,
              |acc, &ref matcher| matcher.replace_all(&acc, "").into_owned());
    Ok(Str(result))
}

/// Removes any newline characters (line breaks) from a string.
pub fn strip_newlines(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let input = input.to_string();
    Ok(Str(input.chars().filter(|c| *c != '\n' && *c != '\r').collect()))
}

pub fn join(input: &Value, args: &[Value]) -> FilterResult {
    if 1 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one, {} given", args.len())));
    }

    let glue = args.get(0)
        .map(|v| v.to_string())
        .unwrap_or_else(|| " ".to_owned());

    let input = input.as_array()
        .ok_or_else(|| InvalidType("Array of strings expected".to_owned()))?;
    // use ToStr to stringify the values in case they aren't strings...
    let input = input.iter().map(|x| x.to_string());

    Ok(Str(itertools::join(input, glue.as_str())))
}

pub fn sort(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    // TODO optional property parameter

    let array = input.as_array().ok_or_else(|| InvalidType("Array expected".to_owned()))?;
    let mut sorted = array.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal));
    Ok(Value::Array(sorted))
}

// TODO sort_natural

/// Removes any duplicate elements in an array.
///
/// This has an O(n^2) worst-case complexity.
pub fn uniq(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    // TODO optional property parameter

    let array = input.as_array()
        .ok_or_else(|| InvalidType("Array expected".to_owned()))?;
    let mut deduped: Vec<Value> = Vec::new();
    for x in array.iter() {
        if !deduped.contains(x) {
            deduped.push(x.clone())
        }
    }
    Ok(Value::Array(deduped))
}

/// Reverses the order of the items in an array. `reverse` cannot `reverse` a string.
pub fn reverse(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let array = input.as_array().ok_or_else(|| InvalidType("Array expected".to_owned()))?;
    let mut reversed = array.clone();
    reversed.reverse();
    Ok(Value::Array(reversed))
}

// TODO map

// TODO compact

pub fn replace(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() < 1 || 2 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one or two, {} given", args.len())));
    }

    let input = input.to_string();

    let search = args[0].to_string();
    let replace = args.get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_owned());

    Ok(Str(input.replace(search.as_str(), replace.as_str())))
}

pub fn replace_first(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() < 1 || 2 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one or two, {} given", args.len())));
    }

    let input = input.to_string();

    let search = args[0].to_string();
    let replace = args.get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_owned());

    {
        let tokens: Vec<&str> = input.splitn(2, search.as_str()).collect();
        if tokens.len() == 2 {
            let result = [tokens[0], replace.as_str(), tokens[1]].join("");
            return Ok(Str(result));
        }
    }
    Ok(Str(input))
}

pub fn remove(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.to_string();

    let string = args[0].to_string();

    Ok(Str(input.replace(string.as_str(), "")))
}

pub fn remove_first(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.to_string();

    let string = args[0].to_string();

    Ok(Str(input.splitn(2, string.as_str()).collect()))
}

pub fn append(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let mut input = input.to_string();

    let string = args[0].to_string();

    input.push_str(string.as_str());

    Ok(Str(input))
}

// TODO concat

pub fn prepend(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.to_string();

    let mut string = args[0].to_string();

    string.push_str(input.as_str());

    Ok(Str(string))
}

/// Replaces every newline (`\n`) with an HTML line break (`<br>`).
pub fn newline_to_br(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    // TODO handle windows line endings
    let input = input.to_string();
    Ok(Str(input.replace("\n", "<br />")))
}

pub fn date(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let format = args[0].to_string();
    if format.is_empty() {
        return Ok(input.clone());
    }

    let input_string = input.as_str().ok_or_else(|| InvalidType("String expected".to_owned()))?;
    let date = DateTime::parse_from_str(input_string, "%d %B %Y %H:%M:%S %z");
    let date = match date {
        Ok(d) => d,
        Err(_) => {
            return Ok(input.clone());
        }
    };

    Ok(Value::Str(date.format(format.as_str()).to_string()))
}

pub fn first(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    match *input {
        Str(ref x) => {
            let c = x.chars().next().map(|c| c.to_string()).unwrap_or_else(|| "".to_owned());
            Ok(Str(c))
        }
        Array(ref x) => Ok(x.first().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
    }
}

pub fn last(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    match *input {
        Str(ref x) => {
            let c = x.chars().last().map(|c| c.to_string()).unwrap_or_else(|| "".to_owned());
            Ok(Str(c))
        }
        Array(ref x) => Ok(x.last().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
    }
}

/// Returns the absolute value of a number.
pub fn abs(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));
    match *input {
        Str(ref s) => {
            match s.parse::<f32>() {
                Ok(n) => Ok(Num(n.abs())),
                Err(e) => {
                    Err(InvalidType(format!("Non-numeric-string, parse error ``{}'' occurred",
                                            e.to_string())))
                }
            }
        }
        Num(n) => Ok(Num(n.abs())),
        _ => Err(InvalidType("String or number expected".to_owned())),
    }
}

pub fn plus(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    let operand = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;

    Ok(Num(input + operand))
}

pub fn minus(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    let operand = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;

    Ok(Num(input - operand))
}

pub fn times(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    let operand = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;

    Ok(Num(input * operand))
}

pub fn divided_by(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    let operand = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;

    // TODO only do `.floor` if its an integer
    Ok(Num((input / operand).floor()))
}

pub fn modulo(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    let operand = args[0].as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;

    Ok(Num(input % operand))
}

pub fn round(input: &Value, args: &[Value]) -> FilterResult {
    if 1 < args.len() {
        return Err(InvalidArgumentCount(format!("expected one, {} given", args.len())));
    }

    let n = args.get(0)
        .unwrap_or(&Value::Num(0_f32))
        .as_float()
        .ok_or_else(|| InvalidArgument(0, "Number expected".to_owned()))?;
    let n = n as usize;

    let input = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;

    if n == 0 {
        Ok(Num(input.round()))
    } else {
        // TODO support this
        Err(InvalidArgument(0,
                            "Rounding to additional places is not yet supported".to_owned()))
    }
}

pub fn ceil(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let n = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;
    Ok(Num(n.ceil()))
}

pub fn floor(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));

    let n = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;
    Ok(Num(n.floor()))
}

pub fn default(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));

    let use_default = match *input {
        Str(ref s) => s.is_empty(),
        Object(ref o) => o.is_empty(),
        Array(ref a) => a.is_empty(),
        Bool(b) => !b,
        Num(_) => false,
    };

    if use_default {
        Ok(args[0].clone())
    } else {
        Ok(input.clone())
    }
}

// shopify

#[cfg(feature = "extra-filters")]
pub fn pluralize(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 2));

    let n = input.as_float().ok_or_else(|| InvalidType("Number expected".to_owned()))?;
    if (n as isize) == 1 {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

// liquid-rust proprietary

#[cfg(feature = "extra-filters")]
pub fn date_in_tz(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 2));

    let s = input.as_str().ok_or_else(|| InvalidType("String expected".to_owned()))?;
    let date = DateTime::parse_from_str(s, "%d %B %Y %H:%M:%S %z")
                .map_err(|e| FilterError::InvalidType(format!("Invalid date format: {}", e)))?;

    let format = args[0].as_str().ok_or_else(|| InvalidArgument(0, "String expected".to_owned()))?;

    let n = args[1].as_float().ok_or_else(|| InvalidArgument(1, "Number expected".to_owned()))?;
    let timezone = FixedOffset::east((n * 3600.0) as i32);

    Ok(Value::Str(date.with_timezone(&timezone).format(format).to_string()))
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use super::*;

    macro_rules! unit {
        ( $a:ident, $b:expr ) => {{
            unit!($a, $b, &[])
        }};
        ( $a:ident, $b:expr , $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    macro_rules! failed {
        ( $a:ident, $b:expr ) => {{
            failed!($a, $b, &[])
        }};
        ( $a:ident, $b:expr, $c:expr ) => {{
            $a(&$b, $c).unwrap_err()
        }};
    }

    macro_rules! tos {
        ( $a:expr ) => {{
            Str($a.to_owned())
        }};
    }

    #[test]
    fn unit_abs() {
        let input = Num(-1f32);
        let args = &[];
        let desired_result = Num(1f32);
        assert_eq!(unit!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_positive_in_string() {
        let input = &tos!("42");
        let args = &[];
        let desired_result = Num(42f32);
        assert_eq!(unit!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_not_number_or_string() {
        let input = &Bool(true);
        let args = &[];
        let desired_result = FilterError::InvalidType("String or number expected".to_owned());
        assert_eq!(failed!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_one_argument() {
        let input = &Num(-1f32);
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(abs, input, args), desired_result);
    }

    #[test]
    fn unit_abs_shopify_liquid() {
        // Three tests from https://shopify.github.io/liquid/filters/abs/
        assert_eq!(unit!(abs, Num(-17f32), &[]), Num(17f32));
        assert_eq!(unit!(abs, Num(4f32), &[]), Num(4f32));
        assert_eq!(unit!(abs, tos!("-19.86"), &[]), Num(19.86f32));
    }

    #[test]
    fn unit_append() {
        assert_eq!(unit!(append, tos!("sam"), &[tos!("son")]), tos!("samson"));
    }

    #[test]
    fn unit_capitalize() {
        assert_eq!(unit!(capitalize, tos!("abc")), tos!("Abc"));
        assert_eq!(unit!(capitalize, tos!("hello world 21")),
                   tos!("Hello world 21"));

        // sure that Umlauts work
        assert_eq!(unit!(capitalize, tos!("über ètat, y̆es?")),
                   tos!("Über ètat, y̆es?"));

        // Weird UTF-8 White space is kept – this is a no-break whitespace!
        assert_eq!(unit!(capitalize, tos!("hello world​")),
                   tos!("Hello world​"));

        // The uppercase version of some character are more than one character long
        assert_eq!(unit!(capitalize, tos!("ßß")), tos!("SSß"));
    }

    #[test]
    fn unit_ceil() {
        assert_eq!(unit!(ceil, Num(1.1f32), &[]), Num(2f32));
        assert_eq!(unit!(ceil, Num(1f32), &[]), Num(1f32));
        assert!(ceil(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_date() {
        assert_eq!(unit!(date,
                         tos!("13 Jun 2016 02:30:00 +0300"),
                         &[tos!("%Y-%m-%d")]),
                   tos!("2016-06-13"));
    }

    #[test]
    fn unit_date_bad_input_type() {
        assert_eq!(failed!(date, Num(0f32), &[tos!("%Y-%m-%d")]),
                   FilterError::InvalidType("String expected".to_owned()));
    }

    #[test]
    fn unit_date_bad_input_format() {
        assert_eq!(unit!(date, tos!("blah blah blah"), &[tos!("%Y-%m-%d")]),
                   tos!("blah blah blah"));
    }

    #[test]
    fn unit_date_format_empty() {
        assert_eq!(unit!(date,
                         tos!("13 Jun 2016 02:30:00 +0300"),
                         &[Str("".to_owned())]),
                   tos!("13 Jun 2016 02:30:00 +0300"));
    }

    #[test]
    fn unit_date_bad_format_type() {
        assert_eq!(unit!(date, tos!("13 Jun 2016 02:30:00 +0300"), &[Num(0f32)]),
                   tos!("0"));
    }

    #[test]
    fn unit_date_missing_format() {
        assert_eq!(failed!(date, tos!("13 Jun 2016 02:30:00 +0300")),
                   FilterError::InvalidArgumentCount("expected 1, 0 given".to_owned()));
    }

    #[test]
    fn unit_date_extra_param() {
        assert_eq!(failed!(date,
                           tos!("13 Jun 2016 02:30:00 +0300"),
                           &[Num(0f32), Num(1f32)]),
                   FilterError::InvalidArgumentCount("expected 1, 2 given".to_owned()));
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_same_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(3f32)];
        let desired_result = tos!("2016-06-13 15:00:00 +0300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_previous_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(-13f32)];
        let desired_result = tos!("2016-06-12 23:00:00 -1300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_next_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(13f32)];
        let desired_result = tos!("2016-06-14 01:00:00 +1300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_input_not_a_string() {
        let input = &Num(0f32);
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(0f32)];
        let desired_result = FilterError::InvalidType("String expected".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_input_not_a_date_string() {
        let input = &tos!("blah blah blah");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(0f32)];
        let desired_result = FilterError::InvalidType("Invalid date format: input contains \
                                                       invalid characters"
            .to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_date_format_not_a_string() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[Num(0f32), Num(1f32)];
        let desired_result = FilterError::InvalidArgument(0, "String expected".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_offset_not_a_num() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), tos!("0")];
        let desired_result = FilterError::InvalidArgument(1, "Number expected".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_zero_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[];
        let desired_result = FilterError::InvalidArgumentCount("expected 2, 0 given".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_one_argument() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z")];
        let desired_result = FilterError::InvalidArgumentCount("expected 2, 1 given".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_three_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Num(0f32), Num(1f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 2, 3 given".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    fn unit_divided_by() {
        assert_eq!(unit!(divided_by, Num(4f32), &[Num(2f32)]), Num(2f32));
        assert_eq!(unit!(divided_by, Num(5f32), &[Num(2f32)]), Num(2f32));
        assert!(divided_by(&Bool(true), &[Num(8.5)]).is_err());
        assert!(divided_by(&Num(2.5), &[Bool(true)]).is_err());
        assert!(divided_by(&Num(2.5), &[]).is_err());
    }

    #[test]
    fn unit_downcase() {
        assert_eq!(unit!(downcase, tos!("Abc")), tos!("abc"));
        assert_eq!(unit!(downcase, tos!("Hello World 21")),
                   tos!("hello world 21"));
    }

    #[test]
    fn unit_escape() {
        assert_eq!(unit!(escape, tos!("Have you read 'James & the Giant Peach'?")),
                   tos!("Have you read &#39;James &amp; the Giant Peach&#39;?"));
        assert_eq!(unit!(escape, tos!("Tetsuro Takara")),
                   tos!("Tetsuro Takara"));
    }

    #[test]
    fn unit_escape_once() {
        assert_eq!(unit!(escape_once, tos!("1 < 2 & 3")),
                   tos!("1 &lt; 2 &amp; 3"));
        assert_eq!(unit!(escape_once, tos!("1 &lt; 2 &amp; 3")),
                   tos!("1 &lt; 2 &amp; 3"));
        assert_eq!(unit!(escape_once, tos!("&lt;&gt;&amp;&#39;&quot;&xyz;")),
                   tos!("&lt;&gt;&amp;&#39;&quot;&amp;xyz;"));
    }

    #[test]
    fn unit_first() {
        assert_eq!(unit!(first,
                         Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])),
                   Num(0f32));
        assert_eq!(unit!(first, Array(vec![tos!("test"), tos!("two")])),
                   tos!("test"));
        assert_eq!(unit!(first, Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_floor() {
        assert_eq!(unit!(floor, Num(1.1f32), &[]), Num(1f32));
        assert_eq!(unit!(floor, Num(1f32), &[]), Num(1f32));
        assert!(floor(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_join() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
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
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[Num(1f32)];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a1b1c"));
    }

    #[test]
    fn unit_join_no_args() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a b c"));
    }

    #[test]
    fn unit_join_non_string_element() {
        let input = Array(vec![tos!("a"), Num(1f32), tos!("c")]);
        let args = &[tos!(",")];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a,1,c"));
    }

    #[test]
    fn unit_last() {
        assert_eq!(unit!(last,
                         Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])),
                   Num(4f32));
        assert_eq!(unit!(last, Array(vec![tos!("test"), tos!("last")])),
                   tos!("last"));
        assert_eq!(unit!(last, Array(vec![])), tos!(""));
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
        let input = &Num(0f32);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(lstrip, input, args), desired_result);
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
    fn unit_minus() {
        assert_eq!(unit!(minus, Num(2f32), &[Num(1f32)]), Num(1f32));
        assert_eq!(unit!(minus, Num(21.5), &[Num(1.25)]), Num(20.25));
    }

    #[test]
    fn unit_modulo() {
        assert_eq!(unit!(modulo, Num(3_f32), &[Num(2_f32)]), Num(1_f32));
        assert_eq!(unit!(modulo, Num(3_f32), &[Num(3.0)]), Num(0_f32));
        assert_eq!(unit!(modulo, Num(24_f32), &[Num(7_f32)]), Num(3_f32));
        assert_eq!(unit!(modulo, Num(183.357), &[Num(12_f32)]), Num(3.3569946));
    }

    #[test]
    fn unit_newline_to_br() {
        let input = &tos!("a\nb");
        let args = &[];
        let desired_result = tos!("a<br />b");
        assert_eq!(unit!(newline_to_br, input, args), desired_result);
    }

    #[test]
    fn unit_newline_to_br_hello_world() {
        // First example from https://shopify.github.io/liquid/filters/newline_to_br/
        let input = &tos!("\nHello\nWorld\n");
        let args = &[];
        let desired_result = tos!("<br />Hello<br />World<br />");
        assert_eq!(unit!(newline_to_br, input, args), desired_result);
    }

    #[test]
    fn unit_newline_to_br_one_argument() {
        let input = &tos!("a\nb");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(newline_to_br, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_pluralize() {
        assert_eq!(unit!(pluralize, Num(1f32), &[tos!("one"), tos!("many")]),
                   tos!("one"));

        assert_eq!(unit!(pluralize, Num(2f32), &[tos!("one"), tos!("many")]),
                   tos!("many"));
    }

    #[test]
    fn unit_plus() {
        assert_eq!(unit!(plus, Num(2f32), &[Num(1f32)]), Num(3f32));
        assert_eq!(unit!(plus, Num(21.5), &[Num(2.25)]), Num(23.75));
    }

    #[test]
    fn unit_prepend() {
        assert_eq!(unit!(prepend, tos!("barbar"), &[tos!("foo")]),
                   tos!("foobarbar"));
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
        assert_eq!(unit!(remove_first, tos!("barbar"), &[tos!("bar")]),
                   tos!("bar"));
        assert_eq!(unit!(remove_first, tos!("barbar"), &[tos!("")]),
                   tos!("barbar"));
        assert_eq!(unit!(remove_first, tos!("barbar"), &[tos!("barbar")]),
                   tos!(""));
        assert_eq!(unit!(remove_first, tos!("barbar"), &[tos!("a")]),
                   tos!("brbar"));
    }

    #[test]
    fn unit_replace() {
        assert_eq!(unit!(replace, tos!("barbar"), &[tos!("bar"), tos!("foo")]),
                   tos!("foofoo"));
    }

    #[test]
    fn unit_replace_first() {
        assert_eq!(unit!(replace_first, tos!("barbar"), &[tos!("bar"), tos!("foo")]),
                   tos!("foobar"));
        assert_eq!(unit!(replace_first, tos!("barxoxo"), &[tos!("xo"), tos!("foo")]),
                   tos!("barfooxo"));
        assert_eq!(unit!(replace_first, tos!(""), &[tos!("bar"), tos!("foo")]),
                   tos!(""));
    }

    #[test]
    fn unit_reverse_apples_oranges_peaches_plums() {
        // First example from https://shopify.github.io/liquid/filters/reverse/
        let input = &Array(vec![tos!("apples"), tos!("oranges"), tos!("peaches"), tos!("plums")]);
        let args = &[];
        let desired_result =
            Array(vec![tos!("plums"), tos!("peaches"), tos!("oranges"), tos!("apples")]);
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_array() {
        let input = &Array(vec![Num(3f32), Num(1f32), Num(2f32)]);
        let args = &[];
        let desired_result = Array(vec![Num(2f32), Num(1f32), Num(3f32)]);
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_array_extra_args() {
        let input = &Array(vec![Num(3f32), Num(1f32), Num(2f32)]);
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_ground_control_major_tom() {
        // Second example from https://shopify.github.io/liquid/filters/reverse/
        let input = &Array(vec![tos!("G"), tos!("r"), tos!("o"), tos!("u"), tos!("n"), tos!("d"),
                                tos!(" "), tos!("c"), tos!("o"), tos!("n"), tos!("t"), tos!("r"),
                                tos!("o"), tos!("l"), tos!(" "), tos!("t"), tos!("o"), tos!(" "),
                                tos!("M"), tos!("a"), tos!("j"), tos!("o"), tos!("r"), tos!(" "),
                                tos!("T"), tos!("o"), tos!("m"), tos!(".")]);
        let args = &[];
        let desired_result = Array(vec![tos!("."), tos!("m"), tos!("o"), tos!("T"), tos!(" "),
                                        tos!("r"), tos!("o"), tos!("j"), tos!("a"), tos!("M"),
                                        tos!(" "), tos!("o"), tos!("t"), tos!(" "), tos!("l"),
                                        tos!("o"), tos!("r"), tos!("t"), tos!("n"), tos!("o"),
                                        tos!("c"), tos!(" "), tos!("d"), tos!("n"), tos!("u"),
                                        tos!("o"), tos!("r"), tos!("G")]);
        assert_eq!(unit!(reverse, input, args), desired_result);
    }

    #[test]
    fn unit_reverse_string() {
        let input = &tos!("abc");
        let args = &[];
        let desired_result = FilterError::InvalidType("Array expected".to_owned());
        assert_eq!(failed!(reverse, input, args), desired_result);
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
        let input = &Num(0f32);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(rstrip, input, args), desired_result);
    }

    #[test]
    fn unit_rstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(rstrip, input, args), desired_result);
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
        assert_eq!(unit!(round, Num(1.1f32), &[]), Num(1f32));
        assert_eq!(unit!(round, Num(1.5f32), &[]), Num(2f32));
        assert_eq!(unit!(round, Num(2f32), &[]), Num(2f32));
        assert!(round(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_size() {
        assert_eq!(unit!(size, tos!("abc")), Num(3f32));
        assert_eq!(unit!(size, tos!("this has 22 characters")), Num(22f32));
        assert_eq!(unit!(size,
                         Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])),
                   Num(5f32));
    }

    #[test]
    fn unit_split() {
        assert_eq!(unit!(split, tos!("a, b, c"), &[tos!(", ")]),
                   Array(vec![tos!("a"), tos!("b"), tos!("c")]));
        assert_eq!(unit!(split, tos!("a~b"), &[tos!("~")]),
                   Array(vec![tos!("a"), tos!("b")]));
    }

    #[test]
    fn unit_split_bad_split_string() {
        let input = tos!("a,b,c");
        let args = &[Num(1f32)];
        let desired_result = Array(vec![tos!("a,b,c")]);
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
        let input = &Num(0f32);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(strip, input, args), desired_result);
    }

    #[test]
    fn unit_strip_one_argument() {
        let input = &tos!(" 	 \n \r test 	 \n \r ");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(strip, input, args), desired_result);
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
    fn unit_strip_html() {
        assert_eq!(unit!(strip_html,
                         tos!("<script type=\"text/javascript\">alert('Hi!');</script>"),
                         &[]),
                   tos!(""));
        assert_eq!(unit!(strip_html,
                         tos!("<SCRIPT type=\"text/javascript\">alert('Hi!');</SCRIPT>"),
                         &[]),
                   tos!(""));
        assert_eq!(unit!(strip_html, tos!("<p>test</p>"), &[]), tos!("test"));
        assert_eq!(unit!(strip_html, tos!("<p id='xxx'>test</p>"), &[]),
                   tos!("test"));
        assert_eq!(unit!(strip_html,
                         tos!("<style type=\"text/css\">cool style</style>"),
                         &[]),
                   tos!(""));
        assert_eq!(unit!(strip_html, tos!("<p\nclass='loooong'>test</p>"), &[]),
                   tos!("test"));
        assert_eq!(unit!(strip_html, tos!("<!--\n\tcomment\n-->test"), &[]),
                   tos!("test"));
        assert_eq!(unit!(strip_html, tos!(""), &[]), tos!(""));
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
        let input = &Num(0f32);
        let args = &[];
        let desired_result = tos!("0");
        assert_eq!(unit!(strip_newlines, input, args), desired_result);
    }

    #[test]
    fn unit_strip_newlines_one_argument() {
        let input = &tos!("ab\n");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(strip_newlines, input, args), desired_result);
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
    fn unit_times() {
        assert_eq!(unit!(times, Num(2f32), &[Num(3f32)]), Num(6f32));
        assert_eq!(unit!(times, Num(8.5), &[Num(0.5)]), Num(4.25));
        assert!(times(&Bool(true), &[Num(8.5)]).is_err());
        assert!(times(&Num(2.5), &[Bool(true)]).is_err());
        assert!(times(&Num(2.5), &[]).is_err());
    }

    #[test]
    fn unit_truncate() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Num(17f32)];
        let desired_result = tos!("I often quote ...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_negative_length() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Num(-17f32)];
        let desired_result = tos!("I often quote myself.  It adds spice to my conversation.");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_non_string() {
        let input = &Num(10000000f32);
        let args = &[Num(5f32)];
        let desired_result = tos!("10...");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_shopify_liquid() {
        // Tests from https://shopify.github.io/liquid/filters/truncate/
        let input = &tos!("Ground control to Major Tom.");
        let args = &[Num(20f32)];
        let desired_result = tos!("Ground control to...");
        assert_eq!(unit!(truncate, input, args), desired_result);

        let args = &[Num(25f32), tos!(", and so on")];
        let desired_result = tos!("Ground control, and so on");
        assert_eq!(unit!(truncate, input, args), desired_result);

        let args = &[Num(20f32), tos!("")];
        let desired_result = tos!("Ground control to Ma");
        assert_eq!(unit!(truncate, input, args), desired_result);
    }

    #[test]
    fn unit_truncate_three_arguments() {
        let input = &tos!("I often quote myself.  It adds spice to my conversation.");
        let args = &[Num(17f32), tos!("..."), Num(0f32)];
        let desired_result =
            FilterError::InvalidArgumentCount("expected one or two arguments, 3 given".to_string());
        assert_eq!(failed!(truncate, input, args), desired_result);
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
        let args = &[Num(20f32)];
        let desired_result = tos!("Here is an a\u{310}, e\u{301}, ...");
        assert_eq!(unit!(truncate, input, args), desired_result);

        // Note that the 🇷🇺🇸🇹 is treated as a single grapheme cluster.
        let input = &tos!("Here is a RUST: 🇷🇺🇸🇹.");
        let args = &[Num(20f32)];
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
        assert_eq!(unit!(truncatewords, tos!("one two three"), &[Num(-1_f32)]),
                   tos!("one two three"));
    }

    #[test]
    fn unit_truncatewords_zero_length() {
        assert_eq!(unit!(truncatewords, tos!("one two three"), &[Num(0_f32)]),
                   tos!("..."));
    }

    #[test]
    fn unit_truncatewords_no_truncation() {
        assert_eq!(unit!(truncatewords, tos!("one two three"), &[Num(4_f32)]),
                   tos!("one two three"));
    }

    #[test]
    fn unit_truncatewords_truncate() {
        assert_eq!(unit!(truncatewords, tos!("one two three"), &[Num(2_f32)]),
                   tos!("one two..."));
        assert_eq!(unit!(truncatewords,
                         tos!("one two three"),
                         &[Num(2_f32), Num(1_f32)]),
                   tos!("one two1"));
    }

    #[test]
    fn unit_truncatewords_empty_string() {
        assert_eq!(unit!(truncatewords, tos!(""), &[Num(1_f32)]), tos!(""));
    }

    #[test]
    fn unit_uniq() {
        let input = &Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        let args = &[];
        let desired_result = Array(vec![tos!("a"), tos!("b")]);
        assert_eq!(unit!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_uniq_non_array() {
        let input = &Num(0f32);
        let args = &[];
        let desired_result = FilterError::InvalidType("Array expected".to_string());
        assert_eq!(failed!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_uniq_one_argument() {
        let input = &Array(vec![tos!("a"), tos!("b"), tos!("a")]);
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_string());
        assert_eq!(failed!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_uniq_shopify_liquid() {
        // Test from https://shopify.github.io/liquid/filters/uniq/
        let input =
            &Array(vec![tos!("ants"), tos!("bugs"), tos!("bees"), tos!("bugs"), tos!("ants")]);
        let args = &[];
        let desired_result = Array(vec![tos!("ants"), tos!("bugs"), tos!("bees")]);
        assert_eq!(unit!(uniq, input, args), desired_result);
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(upcase, tos!("abc")), tos!("ABC"));
        assert_eq!(unit!(upcase, tos!("Hello World 21")),
                   tos!("HELLO WORLD 21"));
    }

    #[test]
    fn unit_default() {
        assert_eq!(unit!(default, tos!(""), &[tos!("bar")]), tos!("bar"));
        assert_eq!(unit!(default, tos!("foo"), &[tos!("bar")]), tos!("foo"));
        assert_eq!(unit!(default, Num(0_f32), &[tos!("bar")]), Num(0_f32));
        assert_eq!(unit!(default, Array(vec![]), &[Num(1_f32)]), Num(1_f32));
        assert_eq!(unit!(default, Array(vec![tos!("")]), &[Num(1_f32)]),
                   Array(vec![tos!("")]));
        assert_eq!(unit!(default, Object(HashMap::new()), &[Num(1_f32)]),
                   Num(1_f32));
        assert_eq!(unit!(default, Bool(false), &[Num(1_f32)]), Num(1_f32));
        assert_eq!(unit!(default, Bool(true), &[Num(1_f32)]), Bool(true));
    }
}
