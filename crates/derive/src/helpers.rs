//! Utility items for this crate.

use std::str::FromStr;
use syn::spanned::Spanned as _;
use syn::{meta, Attribute, Error, LitStr, Path, Result};

/// A wrapper around a type that only allows its value to be assigned once.
#[derive(Debug, Default)]
pub(crate) enum AssignOnce<T> {
    Set(T),
    #[default]
    Unset,
}

impl<T> AssignOnce<T> {
    /// Assigns `value` to `self`, however calls `err` instead if `self` is already assigned.
    pub(crate) fn set<E, F>(&mut self, value: T, err: F) -> std::result::Result<(), E>
    where
        F: FnOnce() -> E,
    {
        match self {
            AssignOnce::Set(_) => Err(err()),
            AssignOnce::Unset => {
                *self = AssignOnce::Set(value);
                Ok(())
            }
        }
    }

    /// Unwraps `self`, returning `default` if `self` is not set.
    pub(crate) fn default_to(self, default: T) -> T {
        match self {
            AssignOnce::Set(value) => value,
            AssignOnce::Unset => default,
        }
    }

    /// Converts this type to `Option`.
    pub(crate) fn into_option(self) -> Option<T> {
        match self {
            AssignOnce::Set(value) => Some(value),
            AssignOnce::Unset => None,
        }
    }

    /// Unwraps `self` or calls `err` if `self` is not set.
    pub(crate) fn unwrap_or_err<E, F>(self, err: F) -> std::result::Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            AssignOnce::Set(value) => Ok(value),
            AssignOnce::Unset => Err(err()),
        }
    }
}

/// Utility function to parse `Meta::NameValue` elements that assigns a String.
pub(crate) fn assign_str_value(
    to: &mut AssignOnce<String>,
    attr: &Attribute,
    key: &str,
    meta: &meta::ParseNestedMeta<'_>,
) -> Result<()> {
    let value = meta.value()?;
    let value: LitStr = value.parse()?;
    to.set(value.value(), || {
        Error::new(
            attr.span(),
            format!("parameter `{key}` was already specified."),
        )
    })
}

/// Utility function to parse `Meta::NameValue` elements that assigns a value parsed from a String.
pub(crate) fn parse_str_value<T>(
    to: &mut AssignOnce<T>,
    attr: &Attribute,
    key: &str,
    meta: &meta::ParseNestedMeta<'_>,
) -> Result<()>
where
    T: FromStr<Err = String>,
{
    let value = meta.value()?;
    let value: LitStr = value.parse::<LitStr>()?;
    let value = value
        .value()
        .parse()
        .map_err(|err| Error::new(attr.span(), err))?;
    to.set(value, || {
        Error::new(
            attr.span(),
            format!("parameter `{key}` was already specified."),
        )
    })
}

/// Utility function to parse `Meta::Word` elements.
pub(crate) fn assign_path(
    to: &mut AssignOnce<Path>,
    attr: &Attribute,
    key: &str,
    meta: &meta::ParseNestedMeta<'_>,
) -> Result<()> {
    meta.parse_nested_meta(|meta| {
        to.set(meta.path, || {
            Error::new(
                attr.span(),
                format!("attribute `{key}` was already specified."),
            )
        })?;
        Ok(())
    })
}
