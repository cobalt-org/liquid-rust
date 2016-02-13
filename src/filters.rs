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
            InvalidType(ref e) => e,
            InvalidArgumentCount(ref e) => e,
            InvalidArgument(_, ref e) => e,
        }
    }
}

pub type FilterResult = Result<String, FilterError>;

pub fn size(input: &Value, _args: &Vec<Value>) -> FilterResult {
    match input {
        &Str(ref s) => Ok(s.len().to_string()),
        &Array(ref x) => Ok(x.len().to_string()),
        &Object(ref x) => Ok(x.len().to_string()),
        _ => Err(InvalidType("String, Array or Object expected".to_string())),
    }
}

pub fn upcase(input: &Value, _args: &Vec<Value>) -> FilterResult {
    match input {
        &Str(ref s) => Ok(s.to_uppercase()),
        _ => Err(InvalidType("String expected".to_string())),
    }
}

pub fn minus(input: &Value, args: &Vec<Value>) -> FilterResult {

    let num = match input {
        &Num(n) => n,
        _ => return Err(InvalidType("Num expected".to_string())),
    };
    match args.first() {
        Some(&Num(x)) => Ok((num - x).to_string()),
        _ => Err(InvalidArgument(0, "Num expected".to_string())),
    }
}

pub fn replace(input: &Value, args: &Vec<Value>) -> FilterResult {
    if args.len() != 2 {
        return Err(InvalidArgumentCount(format!("expected 2, {} given", args.len())));
    }
    match input {
        &Str(ref x) => {
            let arg1 = match &args[0] {
                &Str(ref a) => a,
                _ => return Err(InvalidArgument(0, "Str expected".to_string())),
            };
            let arg2 = match &args[1] {
                &Str(ref a) => a,
                _ => return Err(InvalidArgument(1, "Str expected".to_string())),
            };
            Ok(x.replace(arg1, arg2))
        }
        _ => Err(InvalidType("String expected".to_string())),
    }
}

#[cfg(test)]
mod tests {

    use value::Value::*;
    use super::*;

    macro_rules! unit {
        ( $a:ident, $b:expr ) => {{
            unit!($a, $b, &vec![])
        }};
        ( $a:ident, $b:expr , $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    macro_rules! tos {
        ( $a:expr ) => {{
            Str($a.to_string())
        }};
    }

    #[test]
    fn unit_size() {
        assert_eq!(unit!(size, tos!("abc")), "3");
        assert_eq!(unit!(size, tos!("this has 22 characters")), "22");
    }

    #[test]
    fn unit_upcase() {
        assert_eq!(unit!(upcase, tos!("abc")), "ABC");
        assert_eq!(unit!(upcase, tos!("Hello World 21")), "HELLO WORLD 21");
    }

    #[test]
    fn unit_minus() {
        assert_eq!(unit!(minus, Num(2f32), &vec![Num(1f32)]), "1");
        assert_eq!(unit!(minus, Num(21.5), &vec![Num(1.25)]), "20.25");
    }

    #[test]
    fn unit_replace() {
        assert_eq!(unit!(replace, tos!("barbar"), &vec![tos!("bar"), tos!("foo")]),
                   "foofoo");
    }

}
