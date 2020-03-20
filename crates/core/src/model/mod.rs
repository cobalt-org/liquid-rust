//! Liquid data model.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

pub mod array;
pub mod find;
pub mod object;
pub mod scalar;
pub mod value;

mod ser;

pub use array::{Array, ArrayView};
pub use object::{to_object, Object, ObjectView};
pub use scalar::{Scalar, ScalarCow};
pub use value::{to_value, State, Value, ValueCow, ValueView};
