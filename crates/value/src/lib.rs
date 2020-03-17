//! Liquid Value type.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;

mod array;
mod cow;
mod date;
mod display;
mod map;
mod path;
mod scalar;
mod ser;
mod state;
mod values;
mod view;

pub mod object;

pub use crate::array::*;
pub use crate::cow::*;
pub use crate::date::*;
pub use crate::display::*;
pub use crate::object::{Object, ObjectView};
pub use crate::path::*;
pub use crate::scalar::*;
pub use crate::ser::*;
pub use crate::state::*;
pub use crate::values::*;
pub use crate::view::*;
