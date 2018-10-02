//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;
extern crate chrono;
extern crate liquid_error;
extern crate num_traits;

mod index;
mod scalar;
mod ser;
mod values;

/// Liquid Processing Errors.
pub mod error {
    pub use liquid_error::*;
}

pub use index::*;
pub use scalar::*;
pub use ser::*;
pub use values::*;
