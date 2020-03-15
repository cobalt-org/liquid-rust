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
//! let template = liquid::ParserBuilder::with_stdlib()
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

mod parser;
mod reflection;
mod template;

#[doc(hidden)]
pub use liquid_core::value;
pub use liquid_core::partials;

pub use crate::parser::*;
pub use crate::reflection::*;
pub use crate::template::*;
pub use liquid_core::object;
pub use liquid_core::to_object;
pub use liquid_core::Error;
pub use liquid_core::Object;
pub use liquid_core::{ObjectView, ValueView};
pub use liquid_derive::{ObjectView, ValueView};

#[macro_use]
extern crate doc_comment;
doc_comment! {
    include_str!("../README.md")
}
