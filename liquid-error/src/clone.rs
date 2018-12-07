use std::error;
use std::fmt;

#[derive(Debug)]
pub(crate) enum ClonableError {
    Original(BoxedError),
    Missing,
}

pub(crate) type BoxedError = Box<error::Error + Send + Sync + 'static>;

impl ClonableError {
    pub(crate) fn new(error: BoxedError) -> Self {
        ClonableError::Original(error)
    }

    pub(crate) fn cause(&self) -> Option<&error::Error> {
        match *self {
            ClonableError::Original(ref e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl Clone for ClonableError {
    fn clone(&self) -> Self {
        match *self {
            ClonableError::Original(_) | ClonableError::Missing => ClonableError::Missing,
        }
    }
}

impl fmt::Display for ClonableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClonableError::Original(ref e) => fmt::Display::fmt(e, f),
            ClonableError::Missing => write!(f, "Unknown error"),
        }
    }
}
