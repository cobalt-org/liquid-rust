use std::result;
use std::error;
use std::fmt;

// type alias because we always want to deal with CobaltErrors
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Lexer(String),
    Parser(String),
    Render(String),
    Other(String),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Lexer(ref err) => write!(f, "Syntax error: {}", err),
            Error::Parser(ref err) => write!(f, "Parsing error: {}", err),
            Error::Render(ref err) => write!(f, "Rendering error: {}", err),
            Error::Other(ref err) => write!(f, "Error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Lexer(ref err) => err,
            Error::Parser(ref err) => err,
            Error::Render(ref err) => err,
            Error::Other(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            _ => None,
        }
    }
}
