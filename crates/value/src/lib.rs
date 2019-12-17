//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

mod date;
mod path;
mod scalar;
mod ser;
mod values;

pub mod map;

/// Liquid Processing Errors.
pub mod error {
    pub use liquid_error::*;
}

/// String-type optimized for `Value`
pub mod sstring {
    pub use sstring::*;
}

pub use crate::date::*;
pub use crate::path::*;
pub use crate::scalar::*;
pub use crate::ser::*;
pub use crate::values::*;
