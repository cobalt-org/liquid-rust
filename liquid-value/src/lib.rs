//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

pub mod map;
mod path;
mod scalar;
mod ser;
mod values;

/// Liquid Processing Errors.
pub mod error {
    pub use liquid_error::*;
}

pub use crate::path::*;
pub use crate::scalar::*;
pub use crate::ser::*;
pub use crate::values::*;
