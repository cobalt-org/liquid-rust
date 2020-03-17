//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

mod object;
mod ser;
mod value;

pub mod array;
pub mod find;
pub mod scalar;

pub use crate::object::*;
pub use crate::scalar::{Scalar, ScalarCow};
pub use crate::value::*;
