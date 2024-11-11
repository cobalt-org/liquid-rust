//! ## Using partials
//!
//! To use `{% include %}` or `{% render %}` tags in a template, you first need to compile these
//! included files as template partials.
//!
//! Example:
//!
//! ```liquid
//! # common.liquid
//! Number: {{ i }}
//! ```
//!
//! ```rust
//! use liquid::ParserBuilder;
//! use liquid::partials::{EagerCompiler, InMemorySource, PartialSource};
//!
//! // Build template partials using an eager, in-memory source compiler.
//! // Other compilation policies also exist depending on specific needs.
//! type Partials = EagerCompiler<InMemorySource>;
//!
//! let partials = {
//!   let mut partials = Partials::empty();
//!
//!   let filepath = String::from("common.liquid");
//!   //let contents = std::fs::read_to_string(&filepath).unwrap();
//!   let contents = "Number: {{ i }}";
//!
//!   partials.add(filepath, contents);
//!   partials
//! };
//!
//! // Compile and render the main template, which uses the "common" partial.
//! let parser = ParserBuilder::with_stdlib().partials(partials).build().unwrap();
//! let rendered = {
//!     let globals = liquid::object!({ "num": 42 });
//!     parser
//!         .parse("Liquid! {% render \"common\", i: num %}").unwrap()
//!         .render(&globals).unwrap()
//! };
//!
//! assert_eq!(rendered, "Liquid! Number: 42");
//! ```

pub use liquid_core::partials::*;
