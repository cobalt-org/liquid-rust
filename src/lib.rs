//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.9"
//! ```
//!
//! ## Example
//! ```rust
//! use liquid::{Renderable, Context, Value};
//!
//! let template = liquid::parse("Liquid! {{num | minus: 2}}", Default::default()).unwrap();
//!
//! let mut context = Context::new();
//! context.set_val("num", Value::Num(4f32));
//!
//! let output = template.render(&mut context);
//! assert_eq!(output.unwrap(), Some("Liquid! 2".to_string()));
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

pub use context::Context;
pub use error::Error;
pub use filters::FilterError;
pub use syntax::Renderable;
pub use syntax::Template;
pub use syntax::Token;
pub use syntax::{Value, Object, Array};
pub use syntax::{ParseTag, ParseTagClone, FnParseTag, FnTagParser};
pub use syntax::{ParseBlock, ParseBlockClone, FnParseBlock, FnBlockParser};
pub use syntax::{Include, IncludeClone, NullInclude, FilesystemInclude};

pub mod syntax;

mod context;
mod error;
mod tags;
mod filters;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::path;

use error::Result;

/// Options that `liquid::parse` takes
#[derive(Clone)]
pub struct LiquidOptions {
    /// Holds all custom block-size tags
    pub blocks: HashMap<String, Box<ParseBlock>>,
    /// Holds all custom tags
    pub tags: HashMap<String, Box<ParseTag>>,
    /// The path to which paths in include tags should be relative to
    pub include_source: Box<Include>,
}

impl Default for LiquidOptions {
    fn default() -> LiquidOptions {
        LiquidOptions {
            blocks: Default::default(),
            tags: Default::default(),
            include_source: Box::new(NullInclude::new()),
        }
    }
}

impl LiquidOptions {
    /// Creates a LiquidOptions instance, pre-seeded with all known
    /// tags and blocks.
    pub fn with_known_blocks() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.register_known_blocks();
        options
    }

    /// Registers all known tags and blocks in an existing options
    /// struct
    pub fn register_known_blocks(&mut self) {
        self.register_tag("assign",
                          Box::new(syntax::FnTagParser::new(tags::assign_tag)));
        self.register_tag("break", Box::new(syntax::FnTagParser::new(tags::break_tag)));
        self.register_tag("continue",
                          Box::new(syntax::FnTagParser::new(tags::continue_tag)));
        self.register_tag("cycle", Box::new(syntax::FnTagParser::new(tags::cycle_tag)));
        self.register_tag("include",
                          Box::new(syntax::FnTagParser::new(tags::include_tag)));

        self.register_block("raw", Box::new(syntax::FnBlockParser::new(tags::raw_block)));
        self.register_block("if", Box::new(syntax::FnBlockParser::new(tags::if_block)));
        self.register_block("unless",
                            Box::new(syntax::FnBlockParser::new(tags::unless_block)));
        self.register_block("for", Box::new(syntax::FnBlockParser::new(tags::for_block)));
        self.register_block("comment",
                            Box::new(syntax::FnBlockParser::new(tags::comment_block)));
        self.register_block("capture",
                            Box::new(syntax::FnBlockParser::new(tags::capture_block)));
        self.register_block("case",
                            Box::new(syntax::FnBlockParser::new(tags::case_block)));
    }

    /// Inserts a new custom block into the options object
    pub fn register_block(&mut self, name: &str, block: Box<ParseBlock>) {
        self.blocks.insert(name.to_owned(), block);
    }

    /// Inserts a new custom tag into the options object
    pub fn register_tag(&mut self, name: &str, tag: Box<ParseTag>) {
        self.tags.insert(name.to_owned(), tag);
    }
}

/// Parses a liquid template, returning a Template object.
/// # Examples
///
/// ## Minimal Template
///
/// ```
/// use liquid::{Renderable, LiquidOptions, Context};
///
/// let template = liquid::parse("Liquid!", LiquidOptions::default()).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Liquid!".to_owned()));
/// ```
///
pub fn parse(text: &str, options: LiquidOptions) -> Result<Template> {
    let mut options = options;
    options.register_known_blocks();

    let tokens = syntax::tokenize(text)?;
    syntax::parse(&tokens, &options).map(Template::new)
}

/// Parse a liquid template from a file, returning a `Result<Template, Error>`.
/// # Examples
///
/// ## Minimal Template
///
/// `template.txt`:
///
/// ```text
/// "Liquid {{data}}"
/// ```
///
/// Your rust code:
///
/// ```rust,no_run
/// use liquid::{Renderable, LiquidOptions, Context, Value};
///
/// let template = liquid::parse_file("path/to/template.txt",
///                                   LiquidOptions::default()).unwrap();
/// let mut data = Context::new();
/// data.set_val("data", Value::Num(4f32));
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Liquid 4\n".to_string()));
/// ```
///
pub fn parse_file<P: AsRef<path::Path>>(fp: P, options: LiquidOptions) -> Result<Template> {
    let mut options = options;
    options.register_known_blocks();

    let mut f = File::open(fp)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    parse(&buf, options)
}
