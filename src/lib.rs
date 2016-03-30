#![crate_name = "liquid"]
#![doc(html_root_url = "https://cobalt-org.github.io/liquid-rust/")]

// This library uses Clippy!
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Deny warnings, except in dev mode
#![deny(warnings)]
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
        needless_lifetimes,
        needless_range_loop,
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

use std::collections::HashMap;
use lexer::Element;
use tags::{assign_tag, include_tag, break_tag, continue_tag, comment_block, raw_block, for_block,
           if_block, capture_block};
use std::default::Default;
use error::Result;

pub use value::Value;
pub use context::Context;
pub use template::Template;
pub use error::Error;
pub use filters::{FilterResult, FilterError};
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

/// The ErrorMode to use.
/// This currently does not have an effect, until
/// ErrorModes are properly implemented.
#[derive(Clone, Copy)]
pub enum ErrorMode {
    Strict,
    Warn,
    Lax,
}

impl Default for ErrorMode {
    fn default() -> ErrorMode {
        ErrorMode::Warn
    }
}

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
pub trait Renderable{
    fn render(&self, context: &mut Context) -> Result<Option<String>>;
}

#[derive(Default)]
pub struct LiquidOptions {
    pub blocks: HashMap<String, Box<Block>>,
    pub tags: HashMap<String, Box<Tag>>,
    pub error_mode: ErrorMode,
}

impl LiquidOptions {
    pub fn register_block(&mut self, name: &str, block: Box<Block>) {
        self.blocks.insert(name.to_owned(), block);
    }

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
    let tokens = try!(lexer::tokenize(&text));

    options.register_tag("assign", Box::new(assign_tag));
    options.register_tag("break", Box::new(break_tag));
    options.register_tag("continue", Box::new(continue_tag));
    options.register_tag("include", Box::new(include_tag));

    options.register_block("raw", Box::new(raw_block));
    options.register_block("if", Box::new(if_block));
    options.register_block("for", Box::new(for_block));
    options.register_block("comment", Box::new(comment_block));
    options.register_block("capture", Box::new(capture_block));

    parser::parse(&tokens, &options).map(Template::new)
}
