use token::Token;

use std::result;
use std::error;
use std::fmt;
use std::io;

use filters::FilterError;

// type alias because we always want to deal with CobaltErrors
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Lexer(String),
    Parser(String),
    Render(String),
    Filter(FilterError),
    Other(String),
    Io(io::Error),
}

impl Error {
    pub fn parser<T>(expected: &str, actual: Option<&Token>) -> Result<T> {
        Err(Error::Parser(format!("Expected {}, found {:?}", expected, actual)))
    }

    pub fn renderer<T>(msg: &str) -> Result<T> {
        Err(Error::Render(msg.to_owned()))
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Other(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &'a str) -> Error {
        Error::Other(err.to_owned())
    }
}

impl From<FilterError> for Error {
    fn from(err: FilterError) -> Error {
        Error::Filter(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Lexer(ref err) => write!(f, "Syntax error: {}", err),
            Error::Parser(ref err) => write!(f, "Parsing error: {}", err),
            Error::Render(ref err) => write!(f, "Rendering error: {}", err),
            Error::Filter(ref err) => write!(f, "Filtering error: {}", err),
            Error::Other(ref err) => write!(f, "Error: {}", err),
            Error::Io(ref err) => write!(f, "Io::Error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Lexer(ref err) |
            Error::Parser(ref err) |
            Error::Render(ref err) |
            Error::Other(ref err) => err,
            Error::Filter(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            _ => None,
        }
    }
}
