//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.15"
//! ```
//!
//! ## Example
//! ```rust
//! let template = liquid::ParserBuilder::with_liquid()
//!     .build()
//!     .parse("Liquid! {{num | minus: 2}}").unwrap();
//!
//! let mut globals = liquid::Object::new();
//! globals.insert("num".into(), liquid::Value::scalar(4f64));
//!
//! let output = template.render(&globals).unwrap();
//! assert_eq!(output, "Liquid! 2".to_string());
//! ```
#![crate_name = "liquid"]
#![doc(html_root_url = "https://cobalt-org.github.io/liquid-rust/")]
#![warn(warnings)]
// Allow zero pointers for lazy_static. Otherwise clippy will complain.
#![allow(unknown_lints)]
#![allow(zero_ptr)]

extern crate chrono;
extern crate itertools;
extern crate regex;
extern crate unicode_segmentation;
extern crate url;

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(test)]
extern crate serde_yaml;

extern crate liquid_error;
extern crate liquid_value;
extern crate liquid_interpreter;
extern crate liquid_compiler;

// Minimize retrofits
mod error {
    pub use liquid_error::*;
}
mod value {
    pub use liquid_value::*;
}

mod parser;
mod template;

pub mod compiler {
    pub use liquid_compiler::*;
}
pub mod filters;
pub mod interpreter {
    pub use liquid_interpreter::*;
}
pub mod tags;

pub use error::Error;
pub use parser::{Parser, ParserBuilder};
pub use template::Template;
pub use value::{Array, Date, Index, Object, Scalar, Value};
