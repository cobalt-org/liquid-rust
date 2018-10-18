//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.17"
//! ```
//!
//! ## Example
//! ```rust
//! let template = liquid::ParserBuilder::with_liquid()
//!     .build()
//!     .parse("Liquid! {{num | minus: 2}}").unwrap();
//!
//! let mut globals = liquid::value::Object::new();
//! globals.insert("num".into(), liquid::value::Value::scalar(4f64));
//!
//! let output = template.render(&globals).unwrap();
//! assert_eq!(output, "Liquid! 2".to_string());
//! ```

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

extern crate liquid_compiler;
extern crate liquid_error;
extern crate liquid_interpreter;
extern crate liquid_value;

mod parser;
mod template;

pub mod compiler {
    pub use liquid_compiler::*;
}
pub mod interpreter {
    pub use liquid_interpreter::*;
}
pub mod value {
    pub use liquid_value::*;
}

pub mod filters;
pub mod tags;

pub use interpreter::Globals;
pub use liquid_error::Error;
pub use parser::*;
pub use template::*;
