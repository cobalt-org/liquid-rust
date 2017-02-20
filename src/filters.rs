use std::fmt;
use std::error::Error;
use std::cmp::Ordering;

use value::Value;
use value::Value::{Array, Num, Object, Str};

use chrono::DateTime;

use self::FilterError::*;

use regex::Regex;

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
    match *input {
        Str(ref s) => {
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
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

// Actual filters.

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

pub fn append(input: &Value, args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match args.first() {
                Some(&Str(ref a)) => Ok(Str(format!("{}{}", x, a))),
                _ => Err(InvalidArgument(0, "Str expected".to_owned())),
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn capitalize(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref s) => {
            Ok(Str(s.char_indices().fold(String::new(), |word, (_, chr)| {
                let next_char = match word.chars().last() {
                        Some(last) => {
                            if last.is_whitespace() {
                                chr.to_uppercase().next().unwrap()
                            } else {
                                chr
                            }
                        }
                        _ => chr.to_uppercase().next().unwrap(),
                    }
                    .to_string();
                word + &next_char
            })))
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn ceil(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.ceil())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn date(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));
    let date = match *input {
        Value::Str(ref s) => {
            try!(DateTime::parse_from_str(&s, "%d %B %Y %H:%M:%S %z")
                .map_err(|e| FilterError::InvalidType(format!("Invalid date format: {}", e))))
        }
        _ => return Err(FilterError::InvalidType("String expected".to_owned())),
    };
    let format = match args[0] {
        Value::Str(ref s) => s,
        _ => return Err(InvalidArgument(0, "Str expected".to_owned())),
    };
    Ok(Value::Str(date.format(format).to_string()))
}

pub fn divided_by(input: &Value, args: &[Value]) -> FilterResult {
    let num = match *input {
        Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_owned())),
    };
    match args.first() {
        Some(&Num(x)) => Ok(Num((num / x).floor())),
        _ => Err(InvalidArgument(0, "Num expected".to_owned())),
    }
}

pub fn downcase(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref s) => Ok(Str(s.to_lowercase())),
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn escape(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, false)
}

pub fn escape_once(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, true)
}

pub fn first(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match x.chars().next() {
                Some(c) => Ok(Str(c.to_string())),
                _ => Ok(Str("".to_owned())),
            }
        }
        Array(ref x) => Ok(x.first().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
    }
}

pub fn floor(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.floor())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn join(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));
    match *input {
        Array(ref array) => {
            // use ToStr to stringify the values in case they aren't strings...
            let mut strings_to_join = array.iter().map(|x| x.to_string());
            // the input is in fact an Array of Strings
            match args.first() {  // Check the first (and only) argument
                Some(&Str(ref join_string)) => {
                    // The join string argument is in fact a String
                    let mut result = strings_to_join.next().unwrap_or_else(String::new);
                    for string in strings_to_join {
                        result.push_str(join_string);
                        result.push_str(&string);
                    }
                    Ok(Str(result))
                }
                _ => Err(InvalidArgument(0, "expected String argument as join".to_owned())),
            }
        }
        _ => Err(InvalidType("Array of Strings expected".to_owned())),
    }
}

pub fn last(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match x.chars().last() {
                Some(c) => Ok(Str(c.to_string())),
                _ => Ok(Str("".to_owned())),
            }
        }
        Array(ref x) => Ok(x.last().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
    }
}

/// Removes all whitespaces (tabs, spaces, and newlines) from the beginning of a string.
///
/// The filter does not affect spaces between words.  Note that while this works for the case of
/// tabs, spaces, and newlines, it also removes any other codepoints defined by the Unicode Derived
/// Core Property `White_Space` (per [rust
/// documentation](https://doc.rust-lang.org/std/primitive.str.html#method.trim_left).
pub fn lstrip(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));
    match *input {
        Str(ref s) => Ok(Str(s.trim_left().to_string())),
        _ => return Err(InvalidType("Str expected".to_string())),
    }
}

pub fn minus(input: &Value, args: &[Value]) -> FilterResult {
    let num = match *input {
        Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_owned())),
    };
    match args.first() {
        Some(&Num(x)) => Ok(Num(num - x)),
        _ => Err(InvalidArgument(0, "Num expected".to_owned())),
    }
}

pub fn modulo(input: &Value, args: &[Value]) -> FilterResult {
    let num = match *input {
        Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_owned())),
    };
    match args.first() {
        Some(&Num(x)) => Ok(Num(num % x)),
        _ => Err(InvalidArgument(0, "Num expected".to_owned())),
    }
}

/// Replaces every newline (`\n`) with an HTML line break (`<br>`).
pub fn newline_to_br(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));
    match *input {
        Str(ref x) => Ok(Str(x.replace("\n", "<br />"))),
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn pluralize(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 2));
    match *input {
        Num(1f32) => Ok(args[0].clone()),
        Num(_) => Ok(args[1].clone()),
        _ => Err(InvalidType("Number expected".to_owned())),
    }
}

pub fn plus(input: &Value, args: &[Value]) -> FilterResult {
    let num = match *input {
        Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_owned())),
    };
    match args.first() {
        Some(&Num(x)) => Ok(Num(num + x)),
        _ => Err(InvalidArgument(0, "Num expected".to_owned())),
    }
}

pub fn prepend(input: &Value, args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match args.first() {
                Some(&Str(ref a)) => Ok(Str(format!("{}{}", a, x))),
                _ => Err(InvalidArgument(0, "Str expected".to_owned())),
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn remove(input: &Value, args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match args.first() {
                Some(&Str(ref a)) => Ok(Str(x.replace(a, ""))),
                _ => Err(InvalidArgument(0, "Str expected".to_owned())),
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn remove_first(input: &Value, args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => {
            match args.first() {
                Some(&Str(ref a)) => Ok(Str(x.splitn(2, a).collect())),
                _ => Err(InvalidArgument(0, "Str expected".to_owned())),
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn replace(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 2));
    match *input {
        Str(ref x) => {
            let arg1 = match args[0] {
                Str(ref a) => a,
                _ => return Err(InvalidArgument(0, "Str expected".to_owned())),
            };
            let arg2 = match args[1] {
                Str(ref a) => a,
                _ => return Err(InvalidArgument(1, "Str expected".to_owned())),
            };
            Ok(Str(x.replace(arg1, arg2)))
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn replace_first(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() != 2 {
        return Err(InvalidArgumentCount(format!("expected 2, {} given", args.len())));
    }
    match *input {
        Str(ref x) => {
            let search = match args[0] {
                Str(ref a) => a,
                _ => return Err(InvalidArgument(0, "Str expected".to_owned())),
            };
            let replace = match args[1] {
                Str(ref a) => a,
                _ => return Err(InvalidArgument(1, "Str expected".to_owned())),
            };
            let tokens: Vec<&str> = x.splitn(2, search).collect();
            if tokens.len() == 2 {
                let result = tokens[0].to_string() + replace + tokens[1];
                Ok(Str(result))
            } else {
                Ok(Str(x.to_string()))
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

/// Reverses the order of the items in an array. `reverse` cannot `reverse` a string.
pub fn reverse(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));
    match *input {
        Value::Array(ref array) => {
            let mut reversed = array.clone();
            reversed.reverse();
            Ok(Value::Array(reversed))
        }
        _ => Err(InvalidType("Array argument expected".to_owned())),
    }
}

pub fn round(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.round())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn size(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => Ok(Num(x.len() as f32)),
        Array(ref x) => Ok(Num(x.len() as f32)),
        Object(ref x) => Ok(Num(x.len() as f32)),
        _ => Err(InvalidType("String, Array or Object expected".to_owned())),
    }
}

pub fn slice(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() < 1 || args.len() > 2 {
        return Err(InvalidArgumentCount(format!("expected one or two arguments, {} given",
                                                args.len())));
    }
    let mut start = match args.first() {
        Some(&Num(x)) => x as isize,
        _ => return Err(InvalidArgument(0, "Number expected".to_owned())),
    };
    let mut offset = match args.get(1) {
        Some(&Num(x)) if x > 0f32 => x as isize,
        Some(_) => return Err(InvalidArgument(0, "Positive number expected".to_owned())),
        None => 1,
    };

    match *input {
        Str(ref x) => {
            // this simplifies counting and conversions
            let ilen = x.len() as isize;
            if start > ilen {
                start = ilen;
            }
            // Check for overflows over string length and fallback to allowed values
            if start < 0 {
                start += ilen;
            }
            // start is guaranteed to be positive at this point
            if start + offset > ilen {
                offset = ilen - start;
            }
            Ok(Value::Str(x.chars().skip(start.abs() as usize).take(offset as usize).collect()))
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn sort(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 0));
    match *input {
        Value::Array(ref array) => {
            let mut sorted = array.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            Ok(Value::Array(sorted))
        }
        _ => Err(InvalidType("Array argument expected".to_owned())),
    }
}

pub fn split(input: &Value, args: &[Value]) -> FilterResult {
    try!(check_args_len(args, 1));
    match *input {
        Str(ref string_to_split) => {
            // the input String is in fact a String
            match args.first() { // Check the first (and only) argument
                Some(&Str(ref split_string)) => {
                    // The split string argument is also in fact a String
                    // Split and construct resulting Array
                    Ok(Array(string_to_split.split(split_string)
                        .map(|x| Str(String::from(x)))
                        .collect()))
                }
                _ => Err(InvalidArgument(0, "expected String argument to split".to_owned())),
            }
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn strip_html(input: &Value, _args: &[Value]) -> FilterResult {
    lazy_static! {
        // regexps taken from https://git.io/vXbgS
        static ref MATCHERS: [Regex; 4] = [Regex::new(r"(?is)<script.*?</script>").unwrap(),
                                           Regex::new(r"(?is)<style.*?</style>").unwrap(),
                                           Regex::new(r"(?is)<!--.*?-->").unwrap(),
                                           Regex::new(r"(?is)<.*?>").unwrap()];
    }
    match *input {
        Str(ref x) => {
            let result = MATCHERS.iter()
                .fold(x.to_string(),
                      |acc, &ref matcher| matcher.replace_all(&acc, "").into_owned());
            Ok(Str(result))
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn times(input: &Value, args: &[Value]) -> FilterResult {
    let num = match *input {
        Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_owned())),
    };
    match args.first() {
        Some(&Num(x)) => Ok(Num(num * x)),
        _ => Err(InvalidArgument(0, "Num expected".to_owned())),
    }
}

pub fn truncatewords(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() < 1 || args.len() > 2 {
        return Err(InvalidArgumentCount(format!("expected one or two arguments, {} given",
                                                args.len())));
    }
    let num_words = match args.first() {
        Some(&Num(x)) if x > 0f32 => x as usize,
        _ => return Err(InvalidArgument(0, "Positive number expected".to_owned())),
    };
    let empty = "".to_string();
    let append = match args.get(1) {
        Some(&Str(ref x)) => x,
        _ => &empty,
    };

    match *input {
        Str(ref x) => {
            let words: Vec<&str> = x.split(' ').take(num_words).collect();
            let mut result = words.join(" ");
            if *x != result {
                result = result + append;
            }
            Ok(Str(result))
        }
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn upcase(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref s) => Ok(Str(s.to_uppercase())),
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

#[cfg(test)]
mod tests {

    use value::Value::*;
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
        assert_eq!(unit!(abs, Num(-1f32), &[]), Num(1f32));
    }

    #[test]
    fn unit_abs_positive_in_string() {
        assert_eq!(unit!(abs, tos!("42"), &[]), Num(42f32));
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
                   tos!("Hello World 21"));

        // sure that Umlauts work
        assert_eq!(unit!(capitalize, tos!("über ètat, y̆es?")),
                   tos!("Über Ètat, Y\u{306}es?"));

        // Weird UTF-8 White space is kept – this is a no-break whitespace!
        assert_eq!(unit!(capitalize, tos!("hello world​")),
                   tos!("Hello World​"));

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

        assert_eq!(failed!(date, Num(0f32), &[tos!("%Y-%m-%d")]),
                   FilterError::InvalidType("String expected".to_owned()));

        assert_eq!(failed!(date, tos!("blah blah blah"), &[tos!("%Y-%m-%d")]),
                   FilterError::InvalidType("Invalid date format: input contains invalid \
                                             characters"
                       .to_owned()));

        assert_eq!(failed!(date, tos!("13 Jun 2016 02:30:00 +0300"), &[Num(0f32)]),
                   FilterError::InvalidArgument(0, "Str expected".to_owned()));

        assert_eq!(failed!(date, tos!("13 Jun 2016 02:30:00 +0300")),
                   FilterError::InvalidArgumentCount("expected 1, 0 given".to_owned()));

        assert_eq!(failed!(date,
                           tos!("13 Jun 2016 02:30:00 +0300"),
                           &[Num(0f32), Num(1f32)]),
                   FilterError::InvalidArgumentCount("expected 1, 2 given".to_owned()));
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
        assert!(result.is_err());
    }

    #[test]
    fn unit_join_no_args() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[];
        let result = join(&input, args);
        assert!(result.is_err());
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
    fn unit_lstrip_non_string() {
        let input = &Num(0f32);
        let args = &[];
        let desired_result = FilterError::InvalidType("Str expected".to_string());
        assert_eq!(failed!(lstrip, input, args), desired_result);
    }

    #[test]
    fn unit_lstrip_one_argument() {
        let input = &tos!(" 	 \n \r test");
        let args = &[Num(0f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected 0, 1 given".to_owned());
        assert_eq!(failed!(lstrip, input, args), desired_result);
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
        let desired_result = FilterError::InvalidType("Array argument expected".to_owned());
        assert_eq!(failed!(reverse, input, args), desired_result);
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
    fn unit_split_bad_input() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[tos!(",")];
        let result = split(&input, args);
        assert!(result.is_err());
    }

    #[test]
    fn unit_split_bad_split_string() {
        let input = tos!("a,b,c");
        let args = &[Num(1f32)];
        let result = split(&input, args);
        assert!(result.is_err());
    }

    #[test]
    fn unit_split_no_args() {
        let input = tos!("a,b,c");
        let args = &[];
        let result = split(&input, args);
        assert!(result.is_err());
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
    fn unit_times() {
        assert_eq!(unit!(times, Num(2f32), &[Num(3f32)]), Num(6f32));
        assert_eq!(unit!(times, Num(8.5), &[Num(0.5)]), Num(4.25));
        assert!(times(&Bool(true), &[Num(8.5)]).is_err());
        assert!(times(&Num(2.5), &[Bool(true)]).is_err());
        assert!(times(&Num(2.5), &[]).is_err());
    }

    #[test]
    fn unit_truncatewords() {
        assert_eq!(failed!(truncatewords, tos!("bar bar"), &[Num(-1_f32)]),
                   FilterError::InvalidArgument(0, "Positive number expected".to_owned()));
        assert_eq!(failed!(truncatewords, tos!("bar bar"), &[Num(0_f32)]),
                   FilterError::InvalidArgument(0, "Positive number expected".to_owned()));
        assert_eq!(unit!(truncatewords, tos!("bar bar"), &[Num(1_f32)]),
                   tos!("bar"));
        assert_eq!(unit!(truncatewords, tos!("bar bar"), &[Num(2_f32)]),
                   tos!("bar bar"));
        assert_eq!(unit!(truncatewords, tos!("bar bar"), &[Num(3_f32)]),
                   tos!("bar bar"));
        assert_eq!(unit!(truncatewords, tos!(""), &[Num(1_f32)]), tos!(""));
        assert_eq!(unit!(truncatewords, tos!("bar bar"), &[Num(1_f32), tos!("...")]),
                   tos!("bar..."));
        assert_eq!(unit!(truncatewords, tos!("bar bar"), &[Num(2_f32), tos!("...")]),
                   tos!("bar bar"));
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(upcase, tos!("abc")), tos!("ABC"));
        assert_eq!(unit!(upcase, tos!("Hello World 21")),
                   tos!("HELLO WORLD 21"));
    }
}
