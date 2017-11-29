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

use std::collections::HashMap;
use lexer::Element;
use tags::{assign_tag, cycle_tag, include_tag, break_tag, continue_tag, comment_block, raw_block,
           for_block, if_block, unless_block, capture_block, case_block};
use std::default::Default;
use std::fs::File;
use std::io::prelude::Read;
use std::path::{PathBuf, Path};
use error::Result;

pub use value::Value;
pub use value::Object;
pub use value::Array;
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
mod path;

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [`LiquidOptions`](struct.LiquidOptions.html).
///
/// ## Minimal Example
/// ```
/// # use liquid::{Renderable, LiquidOptions, Context, Error, FnTagParser};
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
/// options.tags.insert(
///     "hello_world".to_owned(),
///     Box::new(FnTagParser::new(|_tag_name, _arguments, _options| {
///         Ok(Box::new(HelloWorld))
///     })),
/// );
///
/// let template = liquid::parse("{{hello_world}}", options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Hello World!".to_owned()));
/// ```
pub trait ParseTag: Send + Sync + ParseTagClone {
    fn parse(&self,
             tag_name: &str,
             arguments: &[Token],
             options: &LiquidOptions)
             -> Result<Box<Renderable>>;
}

pub trait ParseTagClone {
    fn clone_box(&self) -> Box<ParseTag>;
}

impl<T> ParseTagClone for T
    where T: 'static + ParseTag + Clone
{
    fn clone_box(&self) -> Box<ParseTag> {
        Box::new(self.clone())
    }
}

impl Clone for Box<ParseTag> {
    fn clone(&self) -> Box<ParseTag> {
        self.clone_box()
    }
}

pub type FnParseTag = fn(&str, &[Token], &LiquidOptions) -> Result<Box<Renderable>>;

#[derive(Clone)]
pub struct FnTagParser {
    pub parser: FnParseTag,
}

impl FnTagParser {
    pub fn new(parser: FnParseTag) -> Self {
        Self { parser }
    }
}

impl ParseTag for FnTagParser {
    fn parse(&self,
             tag_name: &str,
             arguments: &[Token],
             options: &LiquidOptions)
             -> Result<Box<Renderable>> {
        (self.parser)(tag_name, arguments, options)
    }
}

/// A trait for creating custom custom block-size tags (`{% if something %}{% endif %}`).
/// This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a block and returns
/// a new `Renderable` based on its parameters. The received parameters specify the name
/// of the block, the argument [Tokens](lexer/enum.Token.html) passed to
/// the block, a Vec of all [Elements](lexer/enum.Element.html) inside the block and
/// the global [`LiquidOptions`](struct.LiquidOptions.html).
pub type Block = Fn(&str, &[Token], &[Element], &LiquidOptions) -> Result<Box<Renderable>>;

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable: Send + Sync {
    /// Renders the Renderable instance given a Liquid context.
    /// The Result that is returned signals if there was an error rendering,
    /// the Option<String> that is wrapped by the Result will be None if
    /// the render has run successfully but there is no content to render.
    fn render(&self, context: &mut Context) -> Result<Option<String>>;
}

pub trait TemplateRepository {
    fn read_template(&self, path: &str) -> Result<String>;
}

/// `TemplateRepository` to load files relative to the root
pub struct LocalTemplateRepository {
    root: PathBuf,
}

impl LocalTemplateRepository {
    pub fn new(root: PathBuf) -> LocalTemplateRepository {
        LocalTemplateRepository { root: root }
    }
}

impl TemplateRepository for LocalTemplateRepository {
    fn read_template(&self, relative_path: &str) -> Result<String> {
        let path = self.root.clone().join(relative_path);

        if !path.exists() {
            return Err(Error::from(&*format!("{:?} does not exist", path)));
        }
        let mut file = try!(File::open(path));

        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
}

/// Options that `liquid::parse` takes
pub struct LiquidOptions {
    /// Holds all custom block-size tags
    pub blocks: HashMap<String, Box<Block>>,
    /// Holds all custom tags
    pub tags: HashMap<String, Box<ParseTag>>,
    /// The path to which paths in include tags should be relative to
    pub template_repository: Box<TemplateRepository>,
}

impl Default for LiquidOptions {
    fn default() -> LiquidOptions {
        LiquidOptions {
            blocks: Default::default(),
            tags: Default::default(),
            template_repository: Box::new(LocalTemplateRepository { root: PathBuf::new() }),
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
        self.register_tag("assign", Box::new(FnTagParser::new(assign_tag)));
        self.register_tag("break", Box::new(FnTagParser::new(break_tag)));
        self.register_tag("continue", Box::new(FnTagParser::new(continue_tag)));
        self.register_tag("cycle", Box::new(FnTagParser::new(cycle_tag)));
        self.register_tag("include", Box::new(FnTagParser::new(include_tag)));

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

    let tokens = try!(lexer::tokenize(text));
    parser::parse(&tokens, &options).map(Template::new)
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
pub fn parse_file<P: AsRef<Path>>(fp: P, options: LiquidOptions) -> Result<Template> {
    let mut options = options;
    options.register_known_blocks();

    let mut f = try!(File::open(fp));
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));

    let tokens = try!(lexer::tokenize(&buf));
    parser::parse(&tokens, &options).map(Template::new)
}
