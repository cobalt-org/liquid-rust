use std::fmt;
use std::error::Error;

use value::Value;
use value::Value::*;

use self::FilterError::*;

#[derive(Debug)]
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
        Str(ref s) => Ok(Str(s.char_indices().fold(String::new(), |word, (_, chr)| {
            let next_char = match word.chars().last() {
                Some(last) => match last.is_whitespace() {
                    true => chr.to_uppercase().next().unwrap(),
                    false => chr,
                },
                _ => chr.to_uppercase().next().unwrap(),
            }.to_string();
            word + &next_char
        }))),
        _ => Err(InvalidType("String expected".to_owned())),
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
        Str(ref x) => match args.first() {
                        Some(&Str(ref a)) => Ok(Str(format!("{}{}", a, x))),
                        _ => return Err(InvalidArgument(0, "Str expected".to_owned())),
                    },
        _ => Err(InvalidType("String expected".to_owned())),
    }
}


pub fn append(input: &Value, args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => match args.first() {
                        Some(&Str(ref a)) => Ok(Str(format!("{}{}", x, a))),
                        _ => return Err(InvalidArgument(0, "Str expected".to_owned())),
                    },
        _ => Err(InvalidType("String expected".to_owned())),
    }
}

pub fn first(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => match x.chars().next() {
                Some(c) => Ok(Str(c.to_string())),
                _ => Ok(Str("".to_owned()))
            },
        Array(ref x) => Ok(x.first().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
    }
}


pub fn last(input: &Value, _args: &[Value]) -> FilterResult {
    match *input {
        Str(ref x) => match x.chars().last() {
                Some(c) => Ok(Str(c.to_string())),
                _ => Ok(Str("".to_owned()))
            },
        Array(ref x) => Ok(x.last().unwrap_or(&Str("".to_owned())).to_owned()),
        _ => Err(InvalidType("String or Array expected".to_owned())),
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

    macro_rules! tos {
        ( $a:expr ) => {{
            Str($a.to_owned())
        }};
    }

    #[test]
    fn unit_size() {
        assert_eq!(unit!(size, tos!("abc")), Num(3f32));
        assert_eq!(unit!(size, tos!("this has 22 characters")), Num(22f32));
        assert_eq!(unit!(size, Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])), Num(5f32));
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
        assert_eq!(unit!(append, tos!("sam"), &[tos!("son")]),
                   tos!("samson"));
    }

    #[test]
    fn unit_first() {
        assert_eq!(unit!(first, Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])), Num(0f32));
        assert_eq!(unit!(first, Array(vec![tos!("test"), tos!("two")])), tos!("test"));
        assert_eq!(unit!(first, Array(vec![])), tos!(""));
    }

    #[test]
    fn unit_last() {
        assert_eq!(unit!(last, Array(vec![Num(0f32), Num(1f32), Num(2f32), Num(3f32), Num(4f32)])), Num(4f32));
        assert_eq!(unit!(last, Array(vec![tos!("test"), tos!("last")])), tos!("last"));
        assert_eq!(unit!(last, Array(vec![])), tos!(""));
    }
}
