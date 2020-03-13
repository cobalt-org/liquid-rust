use liquid_core::Error;

#[cfg(feature = "stdlib")]
pub mod std;

#[cfg(feature = "jekyll")]
pub mod jekyll;

#[cfg(feature = "extra")]
pub mod extra;

#[cfg(feature = "shopify")]
pub mod shopify;

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
