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

/// Add context to a `liquid_error::Error`.
pub trait ResultLiquidExt<T> {
    /// Add a new stack frame to the `liquid_error::Error`.
    fn trace<S>(self, trace: S) -> Result<T>
    where
        S: Into<borrow::Cow<'static, str>>;

    /// Add a new stack frame to the `liquid_error::Error`.
    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Add state the current stack frame.
    fn context<K, V>(self, key: K, value: V) -> Result<T>
    where
        K: Into<borrow::Cow<'static, str>>,
        V: Into<borrow::Cow<'static, str>>;

    /// Add state the current stack frame.
    fn context_with<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> (String, String);
}

impl<T> ResultLiquidExt<T> for Result<T> {
    fn trace<S>(self, trace: S) -> Result<T>
    where
        S: Into<borrow::Cow<'static, str>>,
    {
        self.map_err(|err| err.trace(trace))
    }

    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| err.trace(trace()))
    }

    fn context<K, V>(self, key: K, value: V) -> Result<T>
    where
        K: Into<borrow::Cow<'static, str>>,
        V: Into<borrow::Cow<'static, str>>,
    {
        self.map_err(|err| err.context(key, value))
    }

    fn context_with<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> (String, String),
    {
        let (key, value) = context();
        self.map_err(|err| err.context(key, value))
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
            msg,
            user_backtrace: vec![Trace::empty()],
            cause: None,
        };
        Self {
            inner: Box::new(error),
        }
    }

    /// Add a new call to the user-visible backtrace
    pub fn trace<T>(self, trace: T) -> Self
    where
        T: Into<borrow::Cow<'static, str>>,
    {
        self.trace_trace(trace.into())
    }

    fn trace_trace(mut self, trace: borrow::Cow<'static, str>) -> Self {
        let trace = Trace::new(trace);
        self.inner.user_backtrace.push(trace);
        self
    }

    /// Add context to the last traced call.
    ///
    /// Example context: Value that parameters from the `trace` evaluate to.
    pub fn context<K, V>(self, key: K, value: V) -> Self
    where
        K: Into<borrow::Cow<'static, str>>,
        V: Into<borrow::Cow<'static, str>>,
    {
        self.context_cow_string(key.into(), value.into())
    }

    fn context_cow_string(
        mut self,
        key: borrow::Cow<'static, str>,
        value: borrow::Cow<'static, str>,
    ) -> Self {
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
struct Trace {
    trace: Option<borrow::Cow<'static, str>>,
    context: Vec<(borrow::Cow<'static, str>, borrow::Cow<'static, str>)>,
}

impl Trace {
    fn new(trace: borrow::Cow<'static, str>) -> Self {
        Self {
            trace: Some(trace),
            context: vec![],
        }
    }

    fn empty() -> Self {
        Self {
            trace: None,
            context: vec![],
        }
    }

    fn append_context(&mut self, key: borrow::Cow<'static, str>, value: borrow::Cow<'static, str>) {
        self.context.push((key, value));
    }

    fn get_trace(&self) -> Option<&str> {
        self.trace.as_ref().map(|s| s.as_ref())
    }

    fn get_context(&self) -> &[(borrow::Cow<'static, str>, borrow::Cow<'static, str>)] {
        self.context.as_ref()
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
