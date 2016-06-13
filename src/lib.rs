//! The Liquid templating language for Rust
//!
//! __http://liquidmarkup.org/__
//!
//! ```toml
//! [dependencies]
//! liquid = "0.8"
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

// This library uses Clippy!
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Deny warnings, except in dev mode
#![deny(warnings)]
// #![deny(missing_docs)]
#![cfg_attr(feature="dev", warn(warnings))]

// Ignore clippy, except in dev mode
#![cfg_attr(feature="clippy", allow(clippy))]
#![cfg_attr(feature="dev", warn(clippy))]

// Stuff we want clippy to fail on
#![cfg_attr(feature="clippy", deny(
        explicit_iter_loop,
        clone_on_copy,
        len_zero,
        map_clone,
        map_entry,
        match_bool,
        match_same_arms,
        new_ret_no_self,
        new_without_default,
        needless_borrow,
        needless_lifetimes,
        needless_range_loop,
        needless_return,
        no_effect,
        ok_expect,
        out_of_bounds_indexing,
        ptr_arg,
        redundant_closure,
        single_char_pattern,
        unused_collect,
        useless_vec,
        ))]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate chrono;

use std::collections::HashMap;
use lexer::Element;
use tags::{assign_tag, cycle_tag, include_tag, break_tag, continue_tag, comment_block, raw_block,
           for_block, if_block, unless_block, capture_block, case_block};
use std::default::Default;
use std::path::PathBuf;
use error::Result;

pub use value::Value;
pub use context::Context;
pub use template::Template;
pub use error::Error;
pub use filters::FilterError;
pub use token::Token;

pub mod lexer;
pub mod parser;

mod token;
mod error;
mod template;
mod output;
mod text;
mod tags;
mod filters;
mod value;
mod variable;
mod context;

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [LiquidOptions](struct.LiquidOptions.html).
///
/// ## Minimal Example
/// ```
/// # use liquid::{Renderable, LiquidOptions, Context, Error};
///
/// struct HelloWorld;
///
/// impl Renderable for HelloWorld {
///     fn render(&self, _context: &mut Context) -> Result<Option<String>, Error>{
///         Ok(Some("Hello World!".to_owned()))
///     }
/// }
///
/// let mut options : LiquidOptions = Default::default();
/// options.tags.insert("hello_world".to_owned(), Box::new(|_tag_name, _arguments, _options| {
///      Ok(Box::new(HelloWorld))
/// }));
///
/// let template = liquid::parse("{{hello_world}}", options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Hello World!".to_owned()));
/// ```
pub type Tag = Fn(&str, &[Token], &LiquidOptions) -> Result<Box<Renderable>>;

/// A trait for creating custom custom block-size tags (`{% if something %}{% endif %}`). This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a block and returns
/// a new `Renderable` based on its parameters. The received parameters specify the name
/// of the block, the argument [Tokens](lexer/enum.Token.html) passed to
/// the block, a Vec of all [Elements](lexer/enum.Element.html) inside the block and the global [LiquidOptions](struct.LiquidOptions.html).
pub type Block = Fn(&str, &[Token], Vec<Element>, &LiquidOptions) -> Result<Box<Renderable>>;

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable {
    /// Renders the Renderable instance given a Liquid context.
    /// The Result that is returned signals if there was an error rendering,
    /// the Option<String> that is wrapped by the Result will be None if
    /// the render has run successfully but there is no content to render.
    fn render(&self, context: &mut Context) -> Result<Option<String>>;
}

/// Options that liquid::parse takes
#[derive(Default)]
pub struct LiquidOptions {
    /// Holds all custom block-size tags
    pub blocks: HashMap<String, Box<Block>>,
    /// Holds all custom tags
    pub tags: HashMap<String, Box<Tag>>,
    /// The path to which paths in include tags should be relative to
    pub file_system: Option<PathBuf>,
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
        self.register_tag("assign", Box::new(assign_tag));
        self.register_tag("break", Box::new(break_tag));
        self.register_tag("continue", Box::new(continue_tag));
        self.register_tag("cycle", Box::new(cycle_tag));
        self.register_tag("include", Box::new(include_tag));

        self.register_block("raw", Box::new(raw_block));
        self.register_block("if", Box::new(if_block));
        self.register_block("unless", Box::new(unless_block));
        self.register_block("for", Box::new(for_block));
        self.register_block("comment", Box::new(comment_block));
        self.register_block("capture", Box::new(capture_block));
        self.register_block("case", Box::new(case_block));
    }

    /// Inserts a new custom block into the options object
    pub fn register_block(&mut self, name: &str, block: Box<Block>) {
        self.blocks.insert(name.to_owned(), block);
    }

    /// Inserts a new custom tag into the options object
    pub fn register_tag(&mut self, name: &str, tag: Box<Tag>) {
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

    let tokens = try!(lexer::tokenize(&text));
    parser::parse(&tokens, &options).map(Template::new)
}
