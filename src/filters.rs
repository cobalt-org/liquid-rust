use std::fmt;
use std::error::Error;
use std::cmp::Ordering;

use value::Value;
use value::Value::*;

use chrono::DateTime;

use self::FilterError::*;

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

pub fn size(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => Ok(Num(x.len() as f32)),
        Array(ref x) => Ok(Num(x.len() as f32)),
        Object(ref x) => Ok(Num(x.len() as f32)),
        _ => Err(InvalidType("String, Array or Object expected".to_owned())),
    }
}

pub fn upcase(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref s) => Ok(Str(s.to_uppercase())),
        _ => Err(InvalidType("String expected".to_owned())),
    }
}


pub fn downcase(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref s) => Ok(Str(s.to_lowercase())),
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


pub fn pluralize(input: &Value, args: &[Value]) -> FilterResult {

    if args.len() != 2 {
        return Err(InvalidArgumentCount(format!("expected 2, {} given", args.len())));
    }
    match *input {
        Num(1f32) => Ok(args[0].clone()),
        Num(_) => Ok(args[1].clone()),
        _ => Err(InvalidType("Number expected".to_owned())),
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

pub fn floor(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.floor())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn ceil(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.ceil())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn round(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Num(n) => Ok(Num(n.round())),
        _ => Err(InvalidType("Num expected".to_owned())),
    }
}

pub fn replace(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() != 2 {
        return Err(InvalidArgumentCount(format!("expected 2, {} given", args.len())));
    }
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

pub fn split(input: &Value, args: &[Value]) -> FilterResult {
    // Make sure there is only 1 argument to split
    if args.len() != 1 {
        return Err(InvalidArgumentCount(format!("expected 1, {} given", args.len())));
    }


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

pub fn join(input: &Value, args: &[Value]) -> FilterResult {
    // Make sure there is only 1 argument to join
    if args.len() != 1 {
        return Err(InvalidArgumentCount(format!("expected 1, {} given", args.len())));
    }

    match *input {
        Array(ref array) => {
            // use ToStr to stringify the values in case they aren't strings...
            let mut strings_to_join = array.iter().map(|x| x.to_string());
            // the input is in fact an Array of Strings
            match args.first() {  // Check the first (and only) argument
                Some(&Str(ref join_string)) => {
                    // The join string argument is in fact a String
                    let mut result = strings_to_join.next().unwrap_or(String::new());
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

pub fn sort(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() > 0 {
        return Err(InvalidArgumentCount(format!("expected no arguments, {} given", args.len())));
    }
    match input {
        &Value::Array(ref array) => {
            let mut sorted = array.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            Ok(Value::Array(sorted))
        }
        _ => Err(InvalidType("Array argument expected".to_owned())),

    }
}


pub fn date(input: &Value, args: &[Value]) -> FilterResult {
    if args.len() != 1 {
        return Err(FilterError::InvalidArgumentCount(format!("expected 1, {} given", args.len())));
    }

    let date = match input {
        &Value::Str(ref s) => {
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
    fn unit_size() {
        assert_eq!(unit!(size, tos!("abc")), Num(3f32));
        assert_eq!(unit!(size, tos!("this has 22 characters")), Num(22f32));
        assert_eq!(unit!(size,
                         Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])),
                   Num(5f32));
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(upcase, tos!("abc")), tos!("ABC"));
        assert_eq!(unit!(upcase, tos!("Hello World 21")),
                   tos!("HELLO WORLD 21"));
    }

    #[test]
    fn unit_downcase() {
        assert_eq!(unit!(downcase, tos!("Abc")), tos!("abc"));
        assert_eq!(unit!(downcase, tos!("Hello World 21")),
                   tos!("hello world 21"));
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
    fn unit_pluralize() {
        assert_eq!(unit!(pluralize, Num(1f32), &[tos!("one"), tos!("many")]),
                   tos!("one"));

        assert_eq!(unit!(pluralize, Num(2f32), &[tos!("one"), tos!("many")]),
                   tos!("many"));
    }

    #[test]
    fn unit_minus() {
        assert_eq!(unit!(minus, Num(2f32), &[Num(1f32)]), Num(1f32));
        assert_eq!(unit!(minus, Num(21.5), &[Num(1.25)]), Num(20.25));
    }


    #[test]
    fn unit_plus() {
        assert_eq!(unit!(plus, Num(2f32), &[Num(1f32)]), Num(3f32));
        assert_eq!(unit!(plus, Num(21.5), &[Num(2.25)]), Num(23.75));
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
    fn unit_divided_by() {
        assert_eq!(unit!(divided_by, Num(4f32), &[Num(2f32)]), Num(2f32));
        assert_eq!(unit!(divided_by, Num(5f32), &[Num(2f32)]), Num(2f32));
        assert!(divided_by(&Bool(true), &[Num(8.5)]).is_err());
        assert!(divided_by(&Num(2.5), &[Bool(true)]).is_err());
        assert!(divided_by(&Num(2.5), &[]).is_err());
    }

    #[test]
    fn unit_floor() {
        assert_eq!(unit!(floor, Num(1.1f32), &[]), Num(1f32));
        assert_eq!(unit!(floor, Num(1f32), &[]), Num(1f32));
        assert!(floor(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_ceil() {
        assert_eq!(unit!(ceil, Num(1.1f32), &[]), Num(2f32));
        assert_eq!(unit!(ceil, Num(1f32), &[]), Num(1f32));
        assert!(ceil(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_round() {
        assert_eq!(unit!(round, Num(1.1f32), &[]), Num(1f32));
        assert_eq!(unit!(round, Num(1.5f32), &[]), Num(2f32));
        assert_eq!(unit!(round, Num(2f32), &[]), Num(2f32));
        assert!(round(&Bool(true), &[]).is_err());
    }

    #[test]
    fn unit_replace() {
        assert_eq!(unit!(replace, tos!("barbar"), &[tos!("bar"), tos!("foo")]),
                   tos!("foofoo"));
    }

    #[test]
    fn unit_prepend() {
        assert_eq!(unit!(prepend, tos!("barbar"), &[tos!("foo")]),
                   tos!("foobarbar"));
    }

    #[test]
    fn unit_append() {
        assert_eq!(unit!(append, tos!("sam"), &[tos!("son")]), tos!("samson"));
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
    fn unit_last() {
        assert_eq!(unit!(last,
                         Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])),
                   Num(4f32));
        assert_eq!(unit!(last, Array(vec![tos!("test"), tos!("last")])),
                   tos!("last"));
        assert_eq!(unit!(last, Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_split() {
        assert_eq!(unit!(split, tos!("a, b, c"), &[tos!(", ")]),
                   Array(vec![tos!("a"), tos!("b"), tos!("c")]));
        assert_eq!(unit!(split, tos!("a~b"), &[tos!("~")]),
                   Array(vec![tos!("a"), tos!("b")]));
    }

    #[test]
    fn unit_split_no_args() {
        let input = tos!("a,b,c");
        let args = &[];
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
    fn unit_split_bad_input() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[tos!(",")];
        let result = split(&input, args);
        assert!(result.is_err());
    }

    #[test]
    fn unit_join() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[tos!(",")];
        let result = join(&input, args);
        assert_eq!(result.unwrap(), tos!("a,b,c"));
    }

    #[test]
    fn unit_join_no_args() {
        let input = Array(vec![tos!("a"), tos!("b"), tos!("c")]);
        let args = &[];
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
    fn unit_join_bad_input() {
        let input = tos!("a");
        let args = &[tos!(",")];
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
}
