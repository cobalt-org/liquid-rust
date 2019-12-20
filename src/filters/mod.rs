use liquid_error::Error;

pub mod std;

#[cfg(feature = "jekyll-filters")]
pub mod jekyll;

#[cfg(feature = "extra-filters")]
pub mod extra;

pub fn invalid_input<S>(cause: S) -> Error
where
    S: Into<kstring::KString>,
{
    Error::with_msg("Invalid input").context("cause", cause)
}

pub fn invalid_argument<S>(argument: S, cause: S) -> Error
where
    S: Into<kstring::KString>,
{
    Error::with_msg("Invalid argument")
        .context("argument", argument)
        .context("cause", cause)
}
