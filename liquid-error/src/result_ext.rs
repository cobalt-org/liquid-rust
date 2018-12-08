use std::borrow;
use std::error;
use std::result;

use super::Result;
use super::Error;

/// `Result` extension methods for adapting third party errors to `Error`.
pub trait ResultLiquidChainExt<T> {
    /// Create an `Error` with `E` as the cause.
    fn chain(self, msg: &'static str) -> Result<T>;
    /// Create an `Error` with `E` as the cause.
    fn chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultLiquidChainExt<T> for result::Result<T, E>
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

