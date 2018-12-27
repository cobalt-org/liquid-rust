use std::borrow;
use std::error;
use std::fmt;
use std::result;

use super::ErrorClone;
use super::Trace;

/// Convenience type alias for Liquid compiler errors
pub type Result<T> = result::Result<T, Error>;

type BoxedError = Box<ErrorClone>;

/// Compiler error
#[derive(Debug, Clone)]
pub struct Error {
    inner: Box<InnerError>,
}

// Guts of `Error` here to keep `Error`'s memory size small to avoid bloating the size of
// `Result<T>` in the success case and spilling over from register-based returns to stack-based
// returns.  There are already enough memory allocations below, one more
// shouldn't hurt.
#[derive(Debug, Clone)]
struct InnerError {
    msg: borrow::Cow<'static, str>,
    user_backtrace: Vec<Trace>,
    cause: Option<BoxedError>,
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
    pub fn cause<E: ErrorClone>(self, cause: E) -> Self {
        let cause = Box::new(cause);
        self.cause_error(cause)
    }

    fn cause_error(mut self, cause: BoxedError) -> Self {
        let cause = Some(cause);
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
        self.inner.cause.as_ref().and_then(|e| e.cause())
    }
}
