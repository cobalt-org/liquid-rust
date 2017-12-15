use std::error::Error;
use std::fmt;

use syntax::Value;

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
            FilterError::InvalidType(ref e) => write!(f, "Invalid type : {}", e),
            FilterError::InvalidArgumentCount(ref e) => {
                write!(f, "Invalid number of arguments : {}", e)
            }
            FilterError::InvalidArgument(ref pos, ref e) => {
                write!(f, "Invalid argument given at position {} : {}", pos, e)
            }
        }
    }
}

impl Error for FilterError {
    fn description(&self) -> &str {
        match *self {
            FilterError::InvalidType(ref e) |
            FilterError::InvalidArgumentCount(ref e) |
            FilterError::InvalidArgument(_, ref e) => e,
        }
    }
}

pub type FilterResult = Result<Value, FilterError>;
pub type Filter = Fn(&Value, &[Value]) -> FilterResult;
