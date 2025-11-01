use std::error;
use std::fmt;
use std::result;

use super::ErrorClone;
use super::Trace;

/// Convenience type alias for Liquid compiler errors
pub type Result<T, E = Error> = result::Result<T, E>;

type BoxedError = Box<dyn ErrorClone>;

/// Compiler error
#[derive(Debug, Clone)]
pub struct Error {
    inner: Box<InnerError>,
}

impl Error {
    /// Identifies the underlying kind for this error
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }
}

// Guts of `Error` here to keep `Error`'s memory size small to avoid bloating the size of
// `Result<T>` in the success case and spilling over from register-based returns to stack-based
// returns.  There are already enough memory allocations below, one more
// shouldn't hurt.
#[derive(Debug, Clone)]
struct InnerError {
    kind: ErrorKind,
    user_backtrace: Vec<Trace>,
    cause: Option<BoxedError>,
}

impl Error {
    /// Create an error that identifies the provided variable as unknown
    pub fn unknown_variable<S: Into<crate::model::KString>>(name: S) -> Self {
        Self::with_kind(ErrorKind::UnknownVariable).context("requested variable", name)
    }

    /// Create a new error of the given kind
    pub fn with_kind(kind: ErrorKind) -> Self {
        let error = InnerError {
            kind,
            user_backtrace: vec![Trace::empty()],
            cause: None,
        };
        Self {
            inner: Box::new(error),
        }
    }

    /// Create a new custom error with the given message
    pub fn with_msg<S: Into<crate::model::KString>>(msg: S) -> Self {
        Self::with_msg_cow(msg.into())
    }

    fn with_msg_cow(msg: crate::model::KString) -> Self {
        let error = InnerError {
            kind: ErrorKind::Custom(msg),
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
        T: Into<crate::model::KString>,
    {
        self.trace_trace(trace.into())
    }

    fn trace_trace(mut self, trace: crate::model::KString) -> Self {
        let trace = Trace::new(trace);
        self.inner.user_backtrace.push(trace);
        self
    }

    /// Add context to the last traced call.
    ///
    /// Example context: Value that parameters from the `trace` evaluate to.
    pub fn context<K, V>(self, key: K, value: V) -> Self
    where
        K: Into<crate::model::KString>,
        V: Into<crate::model::KString>,
    {
        self.context_cow_string(key.into(), value.into())
    }

    fn context_cow_string(
        mut self,
        key: crate::model::KString,
        value: crate::model::KString,
    ) -> Self {
        self.inner
            .user_backtrace
            .last_mut()
            .expect("always a trace available")
            .append_context(key, value);
        self
    }

    /// Add an external cause to the error for debugging purposes.
    pub fn cause<E: ErrorClone>(self, cause: E) -> Self {
        let cause = Box::new(cause);
        self.cause_error(cause)
    }

    fn cause_error(mut self, cause: BoxedError) -> Self {
        let cause = Some(cause);
        self.inner.cause = cause;
        self
    }

    /// Simplify returning early with an error.
    pub fn into_err<T, E>(self) -> ::std::result::Result<T, E>
    where
        Self: Into<E>,
    {
        let err = self.into();
        Err(err)
    }
}

const ERROR_DESCRIPTION: &str = "liquid";

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", ERROR_DESCRIPTION, self.inner.kind)?;
        for trace in &self.inner.user_backtrace {
            if let Some(trace) = trace.get_trace() {
                writeln!(f, "from: {}", trace)?;
            }
            if !trace.get_context().is_empty() {
                writeln!(f, "  with:")?;
            }
            for (key, value) in trace.get_context() {
                writeln!(f, "    {}={}", key, value)?;
            }
        }
        Ok(())
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.inner.cause.as_ref().and_then(|e| e.source())
    }
}

/// The type of an error.
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// A variable was being indexed but the desired index did not exist
    UnknownIndex,
    /// A referenced variable did not exist
    UnknownVariable,
    /// A custom error with no discernible kind
    Custom(crate::model::KString),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnknownIndex => f.write_str("Unknown index"),
            ErrorKind::UnknownVariable => f.write_str("Unknown variable"),
            ErrorKind::Custom(s) => s.fmt(f),
        }
    }
}
