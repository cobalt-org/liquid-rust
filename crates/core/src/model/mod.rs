//! Liquid data model.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
mod macros;

mod object;
mod ser;
mod value;

pub mod array;
pub mod find;
pub mod scalar;

pub use object::*;
pub use scalar::{Scalar, ScalarCow};
pub use value::*;
