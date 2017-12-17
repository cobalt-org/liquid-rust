//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.13"
//! ```
//!
//! ## Example
//! ```rust
//! let template = liquid::ParserBuilder::with_liquid()
//!     .build()
//!     .parse("Liquid! {{num | minus: 2}}").unwrap();
//!
//! let mut globals = liquid::Object::new();
//! globals.insert("num".to_owned(), liquid::Value::Num(4f32));
//!
//! let output = template.render(&globals).unwrap();
//! assert_eq!(output, "Liquid! 2".to_string());
//! ```
#![crate_name = "liquid"]
#![doc(html_root_url = "https://cobalt-org.github.io/liquid-rust/")]

// Deny warnings, except in dev mode
#![deny(warnings)]
// #![deny(missing_docs)]
#![cfg_attr(feature="dev", warn(warnings))]

// Allow zero pointers for lazy_static. Otherwise clippy will complain.
#![allow(unknown_lints)]
#![allow(zero_ptr)]

extern crate regex;
extern crate chrono;
extern crate unicode_segmentation;
extern crate itertools;
extern crate url;

#[macro_use]
extern crate lazy_static;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[cfg(test)]
extern crate serde_yaml;

mod error;
mod filters;
mod parser;
mod tags;
mod template;
mod value;

pub mod compiler;
pub mod interpreter;

pub use parser::{ParserBuilder, Parser};
pub use template::Template;
pub use error::Error;
pub use value::{Value, Object, Array, Index};
