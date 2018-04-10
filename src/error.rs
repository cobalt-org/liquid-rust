use std::borrow;
use std::error;
use std::fmt;
use std::result;

/// Convenience type alias for Liquid compiler errors
pub type Result<T> = result::Result<T, Error>;

/// `Result` extension methods for adapting third party errors to `Error`.
pub trait ResultLiquidChainExt<T, E> {
    /// Create an `Error` with `E` as the cause.
    fn chain(self, msg: &'static str) -> Result<T>;
    /// Create an `Error` with `E` as the cause.
    fn chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

/// `Result` convenience extension methods for working with `Error`.
impl<T, E> ResultLiquidChainExt<T, E> for result::Result<T, E>
where
    E: error::Error + Send + Sync + 'static,
{
    fn chain(self, msg: &'static str) -> Result<T> {
        self.map_err(|err| Error::with_msg(msg).cause(err))
    }

    fn chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| Error::with_msg(msg()).cause(err))
    }
}

pub trait ResultLiquidExt<T> {
    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> Trace;
    fn context<S>(self, key: &'static str, value: &S) -> Result<T>
    where
        S: ToString;
    fn context_with<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> (borrow::Cow<'static, str>, String);
}

impl<T> ResultLiquidExt<T> for Result<T> {
    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> Trace,
    {
        self.map_err(|err| err.trace(trace()))
    }

    fn context<S>(self, key: &'static str, value: &S) -> Result<T>
    where
        S: ToString,
    {
        self.map_err(|err| err.context(key, value))
    }

    fn context_with<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> (borrow::Cow<'static, str>, String),
    {
        let (key, value) = context();
        self.map_err(|err| err.context(key, &value))
    }
}

/// Compiler error
#[derive(Clone, Debug)]
pub struct Error {
    inner: Box<InnerError>,
}

// Guts of `Error` here to keep `Error` small to avoid bloating the size of `Result<T>` in the
// success case.  There are already enough memory allocations below, one more shouldn't hurt.
#[derive(Clone, Debug)]
struct InnerError {
    msg: borrow::Cow<'static, str>,
    user_backtrace: Vec<Trace>,
    cause: Option<ErrorCause>,
}

impl Error {
    /// Create a new compiler error with the given message
    pub fn with_msg<S: Into<borrow::Cow<'static, str>>>(msg: S) -> Self {
        Self::with_msg_cow(msg.into())
    }

    fn with_msg_cow(msg: borrow::Cow<'static, str>) -> Self {
        let error = InnerError {
            msg: msg,
            user_backtrace: vec![Trace::empty()],
            cause: None,
        };
        Self {
            inner: Box::new(error),
        }
    }

    /// Add a new call to the user-visible backtrace
    pub fn trace<T: Into<Trace>>(self, trace: T) -> Self {
        self.trace_trace(trace.into())
    }

    fn trace_trace(mut self, trace: Trace) -> Self {
        self.inner.user_backtrace.push(trace);
        self
    }

    /// Add context to the last traced call.
    ///
    /// Example context: Value that parameters from ehe `trace` evaluate to.
    pub fn context<C, S>(self, key: C, value: &S) -> Self
    where
        C: Into<borrow::Cow<'static, str>>,
        S: ToString,
    {
        self.context_cow_string(key.into(), value.to_string())
    }

    fn context_cow_string(mut self, key: borrow::Cow<'static, str>, value: String) -> Self {
        self.inner
            .user_backtrace
            .last_mut()
            .expect("always a trace available")
            .append_context(key, value);
        self
    }

    /// Add an external cause to the error for debugging purposes.
    pub fn cause<E: error::Error + Send + Sync + 'static>(self, cause: E) -> Self {
        let cause = Box::new(cause);
        self.cause_error(cause)
    }

    fn cause_error(mut self, cause: Box<error::Error + Send + Sync + 'static>) -> Self {
        let cause = Some(ErrorCause::Generic(GenericError(cause)));
        self.inner.cause = cause;
        self
    }
}

const ERROR_DESCRIPTION: &str = "liquid";

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}: {}", ERROR_DESCRIPTION, self.inner.msg)?;
        if let Some(ref cause) = self.inner.cause {
            writeln!(f, "cause: {}", cause)?;
        }
        for trace in &self.inner.user_backtrace {
            if let Some(trace) = trace.get_trace() {
                writeln!(f, "from: {}", trace)?;
            }
            if !trace.get_context().is_empty() {
                writeln!(f, "  with:")?;
            }
            for &(ref key, ref value) in trace.get_context() {
                writeln!(f, "    {}={}", key, value)?;
            }
        }
        Ok(())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        ERROR_DESCRIPTION
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.inner.cause {
            Some(ErrorCause::Generic(ref e)) => Some(e.0.as_ref()),
            _ => None,
        }
    }
}

/// User-visible call trace
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Trace {
    trace: Option<String>,
    context: Vec<(borrow::Cow<'static, str>, String)>,
}

impl Trace {
    /// User-visible call trace.
    pub fn new(trace: String) -> Self {
        Self {
            trace: Some(trace),
            context: vec![],
        }
    }

    /// Add context to the traced call.
    ///
    /// Example context: Value that parameters from ehe `trace` evaluate to.
    pub fn context(mut self, key: borrow::Cow<'static, str>, value: String) -> Self {
        self.context.push((key, value));
        self
    }

    pub(self) fn empty() -> Self {
        Self {
            trace: None,
            context: vec![],
        }
    }

    pub(self) fn append_context(&mut self, key: borrow::Cow<'static, str>, value: String) {
        self.context.push((key, value));
    }

    pub fn get_trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| s.as_ref())
    }

    pub fn get_context(&self) -> &[(borrow::Cow<'static, str>, String)] {
        self.context.as_ref()
    }
}

impl From<String> for Trace {
    fn from(trace: String) -> Self {
        Self::new(trace)
    }
}

#[derive(Debug)]
enum ErrorCause {
    Generic(GenericError),
    Missing,
}

impl Clone for ErrorCause {
    fn clone(&self) -> Self {
        match *self {
            ErrorCause::Generic(_) | ErrorCause::Missing => ErrorCause::Missing,
        }
    }
}

impl fmt::Display for ErrorCause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCause::Generic(ref e) => fmt::Display::fmt(e, f),
            ErrorCause::Missing => write!(f, "Unknown error"),
        }
    }
}

struct GenericError(Box<error::Error + Send + Sync + 'static>);

impl fmt::Debug for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
