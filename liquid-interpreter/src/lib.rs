//! Liquid template language interpreter.

#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate liquid_error;
extern crate liquid_value;

#[cfg(test)]
extern crate serde_yaml;

/// Liquid Processing Errors.
pub mod error {
    pub use liquid_error::*;
}
/// Liquid value type.
pub mod value {
    pub use liquid_value::*;
}

mod argument;
mod context;
mod filter;
mod globals;
mod output;
mod renderable;
mod template;
mod text;
mod variable;

pub use self::argument::*;
pub use self::context::*;
pub use self::filter::*;
pub use self::globals::*;
pub use self::output::*;
pub use self::renderable::*;
pub use self::template::*;
pub use self::text::*;
pub use self::variable::*;
