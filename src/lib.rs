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
//! let mut globals = liquid::value::Object::new();
//! globals.insert("num".into(), liquid::value::Value::scalar(4f64));
//!
//! let output = template.render(&globals).unwrap();
//! assert_eq!(output, "Liquid! 2".to_string());
//! ```

extern crate chrono;
extern crate deunicode;
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

extern crate liquid_compiler;
extern crate liquid_derive;
extern crate liquid_error;
extern crate liquid_interpreter;
extern crate liquid_value;

mod parser;
mod template;

/// Allows `liquid-derive` macros to work inside this crate.
///
/// This is necessary because paths to liquid items will
/// start with `::liquid` in those macros.
mod liquid {
    pub use *;
}

pub mod compiler {
    pub use liquid_compiler::*;
}
pub mod error {
    pub use liquid_error::*;
}
pub mod interpreter {
    pub use liquid_interpreter::*;
}
pub mod value {
    pub use liquid_value::*;
}
pub mod derive {
    pub use liquid_derive::*;
}

pub mod filters;
pub mod partials;
pub mod tags;

pub use interpreter::ValueStore;
pub use liquid_error::Error;
pub use parser::*;
pub use template::*;
