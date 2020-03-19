//! Liquid data model.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
mod macros;

mod ser;

pub mod array;
pub mod find;
pub mod object;
pub mod scalar;
pub mod value;

pub use object::{to_object, Object, ObjectView};
pub use scalar::{Scalar, ScalarCow};
pub use value::{to_value, State, Value, ValueCow, ValueView};
