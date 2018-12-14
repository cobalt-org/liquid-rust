//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;
extern crate chrono;
extern crate itertools;
extern crate liquid_error;
extern crate num_traits;

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

pub use path::*;
pub use scalar::*;
pub use ser::*;
pub use values::*;
