//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;
extern crate chrono;

mod index;
mod scalar;
mod values;

pub use index::*;
pub use scalar::*;
pub use values::*;
