use std::borrow;
use std::error;
use std::result;

use super::CloneableError;
use super::Error;
use super::ErrorClone;
use super::Result;

type CowStr = borrow::Cow<'static, str>;

/// `Result` extension methods for adapting third party errors to `Error`.
pub trait ResultLiquidChainExt<T> {
    /// Create an `Error` with `E` as the cause.
    #[must_use]
    fn chain<S: Into<CowStr>>(self, msg: S) -> Result<T>;

    /// Create an `Error` with `E` as the cause.
    #[must_use]
    fn chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr;
}

/// `Result` extension methods for adapting third party errors to `Error`.
pub trait ResultLiquidReplaceExt<T> {
    /// Create an `Error` ignoring `E` as the cause.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidReplaceExt;
    ///
    /// let error = Err(io::Error::new(io::ErrorKind::NotFound, "Oops"));
    /// let error: Result<i32> = error.lossy_chain("Missing liquid partial");
    /// ```
    #[must_use]
    fn lossy_chain<S: Into<CowStr>>(self, msg: S) -> Result<T>;

    /// Create an `Error` ignoring `E` as the cause.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidReplaceExt;
    ///
    /// let filename = "foo";
    /// let error = Err(io::Error::new(io::ErrorKind::NotFound, "Oops"));
    /// let error: Result<i32> = error
    ///     .lossy_chain_with(|| format!("Missing liquid partial: {}", filename).into());
    /// ```
    #[must_use]
    fn lossy_chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr;

    /// Create an `Error` ignoring `E` as the cause.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidReplaceExt;
    ///
    /// let error = Err(io::Error::new(io::ErrorKind::NotFound, "Oops"));
    /// let error: Result<i32> = error.replace("Missing liquid partial");
    /// ```
    #[must_use]
    fn replace<S: Into<CowStr>>(self, msg: S) -> Result<T>;

    /// Create an `Error` ignoring `E` as the cause.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidReplaceExt;
    ///
    /// let filename = "foo";
    /// let error = Err(io::Error::new(io::ErrorKind::NotFound, "Oops"));
    /// let error: Result<i32> = error
    ///     .replace_with(|| format!("Missing liquid partial: {}", filename).into());
    /// ```
    #[must_use]
    fn replace_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr;
}

impl<T, E> ResultLiquidChainExt<T> for result::Result<T, E>
where
    E: ErrorClone,
{
    fn chain<S: Into<CowStr>>(self, msg: S) -> Result<T> {
        self.map_err(|err| Error::with_msg(msg).cause(err))
    }

    fn chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr,
    {
        self.map_err(|err| Error::with_msg(msg()).cause(err))
    }
}

impl<T, E> ResultLiquidReplaceExt<T> for result::Result<T, E>
where
    E: error::Error + Send + Sync + 'static,
{
    fn lossy_chain<S: Into<CowStr>>(self, msg: S) -> Result<T> {
        self.map_err(|err| Error::with_msg(msg).cause(CloneableError::new(err)))
    }

    fn lossy_chain_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr,
    {
        self.map_err(|err| Error::with_msg(msg()).cause(CloneableError::new(err)))
    }

    fn replace<S: Into<CowStr>>(self, msg: S) -> Result<T> {
        self.map_err(|_| Error::with_msg(msg))
    }

    fn replace_with<F>(self, msg: F) -> Result<T>
    where
        F: FnOnce() -> CowStr,
    {
        self.map_err(|_| Error::with_msg(msg()))
    }
}

/// Add context to a `liquid_error::Error`.
pub trait ResultLiquidExt<T>
where
    Self: ::std::marker::Sized,
{
    /// Add a new stack frame to the `liquid_error::Error`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use liquid_error::Error;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidExt;
    ///
    /// let error: Result<i32> = Err(Error::with_msg("Oops"));
    /// let error = error.trace("Within forloop");
    /// ```
    #[must_use]
    fn trace<S>(self, trace: S) -> Result<T>
    where
        S: Into<CowStr>;

    /// Add a new stack frame to the `liquid_error::Error`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use liquid_error::Error;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidExt;
    ///
    /// let for_var = "foo";
    /// let error: Result<i32> = Err(Error::with_msg("Oops"));
    /// let error = error.trace_with(|| format!("Within forloop with {}", for_var).into());
    /// ```
    #[must_use]
    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> CowStr;

    /// Add state the current stack frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use liquid_error::Error;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidExt;
    ///
    /// let for_var = "foo";
    /// let error: Result<i32> = Err(Error::with_msg("Oops"));
    /// let error = error
    ///     .context_key("foo")
    ///     .value("10");
    /// let error = error
    ///     .context_key("foo")
    ///     .value_with(|| format!("{}", for_var).into());
    /// ```
    #[must_use]
    fn context_key<S>(self, key: S) -> Key<T>
    where
        S: Into<CowStr>;

    /// Add state the current stack frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use liquid_error::Error;
    /// use liquid_error::Result;
    /// use liquid_error::ResultLiquidExt;
    ///
    /// let for_var = "foo";
    /// let error: Result<i32> = Err(Error::with_msg("Oops"));
    /// let error = error
    ///     .context_key_with(|| format!("{}", 10).into())
    ///     .value("10");
    /// let error = error
    ///     .context_key_with(|| format!("{}", 10).into())
    ///     .value_with(|| format!("{}", for_var).into());
    /// ```
    #[must_use]
    fn context_key_with<F>(self, key: F) -> FnKey<T, F>
    where
        F: FnOnce() -> CowStr;
}

impl<T> ResultLiquidExt<T> for Result<T> {
    fn trace<S>(self, trace: S) -> Result<T>
    where
        S: Into<CowStr>,
    {
        self.map_err(|err| err.trace(trace))
    }

    fn trace_with<F>(self, trace: F) -> Result<T>
    where
        F: FnOnce() -> CowStr,
    {
        self.map_err(|err| err.trace(trace()))
    }

    fn context_key<S>(self, key: S) -> Key<T>
    where
        S: Into<CowStr>,
    {
        Key::new(self, key)
    }

    fn context_key_with<F>(self, key: F) -> FnKey<T, F>
    where
        F: FnOnce() -> CowStr,
    {
        FnKey::new(self, key)
    }
}

/// Partially constructed context (missing value) for `Result<T>`.
#[allow(missing_debug_implementations)]
pub struct Key<T> {
    builder: Result<T>,
    key: CowStr,
}

impl<T> Key<T> {
    /// Save off a key for a context that will be added to `builder`.
    #[must_use]
    pub fn new<S>(builder: Result<T>, key: S) -> Self
    where
        S: Into<CowStr>,
    {
        Self {
            builder,
            key: key.into(),
        }
    }

    /// Finish creating context and add it to `Result<T>`.
    #[must_use]
    pub fn value<S>(self, value: S) -> Result<T>
    where
        S: Into<CowStr>,
    {
        let builder = self.builder;
        let key = self.key;
        builder.map_err(|err| err.context(key, value.into()))
    }

    /// Finish creating context and add it to `Result<T>`.
    #[must_use]
    pub fn value_with<F>(self, value: F) -> Result<T>
    where
        F: FnOnce() -> CowStr,
    {
        let builder = self.builder;
        let key = self.key;
        builder.map_err(|err| err.context(key, value()))
    }
}

/// Partially constructed context (missing value) for `Result<T>`.
#[allow(missing_debug_implementations)]
pub struct FnKey<T, F>
where
    F: FnOnce() -> CowStr,
{
    builder: Result<T>,
    key: F,
}

impl<T, F> FnKey<T, F>
where
    F: FnOnce() -> CowStr,
{
    /// Save off a key for a context that will be added to `builder`.
    #[must_use]
    pub fn new(builder: Result<T>, key: F) -> Self {
        Self { builder, key }
    }

    /// Finish creating context and add it to `Result<T>`.
    #[must_use]
    pub fn value<S>(self, value: S) -> Result<T>
    where
        S: Into<CowStr>,
    {
        let builder = self.builder;
        let key = self.key;
        builder.map_err(|err| err.context((key)(), value.into()))
    }

    /// Finish creating context and add it to `Result<T>`.
    #[must_use]
    pub fn value_with<V>(self, value: V) -> Result<T>
    where
        V: FnOnce() -> CowStr,
    {
        let builder = self.builder;
        let key = self.key;
        builder.map_err(|err| err.context((key)(), value()))
    }
}
