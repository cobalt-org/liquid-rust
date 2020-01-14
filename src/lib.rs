//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.18"
//! ```
//!
//! ## Example
//! ```rust
//! let template = liquid::ParserBuilder::with_liquid()
//!     .build().unwrap()
//!     .parse("Liquid! {{num | minus: 2}}").unwrap();
//!
//! let mut globals = liquid::object!({
//!     "num": 4f64
//! });
//!
//! let output = template.render(&globals).unwrap();
//! assert_eq!(output, "Liquid! 2".to_string());
//! ```

extern crate chrono;
extern crate deunicode;
extern crate itertools;
extern crate once_cell;
extern crate regex;
extern crate unicode_segmentation;
extern crate url;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(test)]
extern crate serde_yaml;

extern crate kstring;
extern crate liquid_core;

mod parser;
mod reflection;
mod template;

pub mod filters;
pub mod partials;
pub mod tags;

pub use liquid_core::object;
pub use liquid_core::to_object;
pub use liquid_core::Error;
pub use liquid_core::Object;
pub use liquid_core::{ObjectView, ValueView};
pub use parser::*;
pub use reflection::*;
pub use template::*;

#[macro_use]
extern crate doc_comment;
doc_comment! {
    include_str!("../README.md")
}
