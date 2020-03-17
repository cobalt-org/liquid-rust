//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

mod cow;
mod display;
mod ser;
mod state;
mod values;
mod view;

pub mod array;
pub mod find;
pub mod object;
pub mod scalar;

pub use crate::cow::*;
pub use crate::display::*;
pub use crate::object::{to_object, Object, ObjectView};
pub use crate::scalar::{Scalar, ScalarCow};
pub use crate::ser::to_value;
pub use crate::state::*;
pub use crate::values::*;
pub use crate::view::*;
