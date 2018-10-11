//! Liquid template language interpreter.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

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

mod context;
mod expression;
mod filter;
mod filter_chain;
mod globals;
mod renderable;
mod template;
mod text;
mod variable;

pub use self::context::*;
pub use self::expression::*;
pub use self::filter::*;
pub use self::filter_chain::*;
pub use self::globals::*;
pub use self::renderable::*;
pub use self::template::*;
pub use self::text::*;
pub use self::variable::*;
