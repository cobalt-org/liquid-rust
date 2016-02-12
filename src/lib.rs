#![crate_name = "liquid"]
#![doc(html_root_url = "https://cobalt-org.github.io/liquid-rust/")]

#![deny(warnings)]

extern crate regex;

use std::collections::HashMap;
use lexer::Token;
use lexer::Element;
use tags::{comment_block, raw_block, for_block, if_block};
use std::default::Default;
use error::Result;

pub use value::Value;
pub use context::Context;
pub use template::Template;
pub use error::Error;

pub mod lexer;
pub mod parser;

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
pub enum ErrorMode{
    Strict,
    Warn,
    Lax
}

impl Default for ErrorMode {
   fn default() -> ErrorMode { ErrorMode::Warn }
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
/// # use std::default::Default;
/// # use liquid::Renderable;
/// # use liquid::LiquidOptions;
/// # use liquid::Context;
/// # use liquid::Error;
/// # use liquid::Tag;
/// # use liquid::lexer::Token;
/// struct HelloWorld;
///
/// impl Renderable for HelloWorld {
///     fn render(&self, _context: &mut Context) -> Result<Option<String>, Error>{
///         Ok(Some("Hello World!".to_owned()))
///     }
/// }
///
/// fn hw_block(tag_name: &str, arguments: &[Token], options: &LiquidOptions) -> Box<Renderable> {
///      Box::new(HelloWorld) as Box<Renderable>
/// }
///
/// let mut options : LiquidOptions = Default::default();
/// options.tags.insert("hello_world".to_owned(), Box::new(hw_block) as Box<Tag>);
///
/// let template = liquid::parse("{{hello_world}}", options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Hello World!".to_owned()));
/// ```
pub type Tag = Fn(&str, &[Token], &LiquidOptions) -> Box<Renderable>;

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
    pub blocks : HashMap<String, Box<Block>>,
    pub tags : HashMap<String, Box<Tag>>,
    pub error_mode : ErrorMode
}

/// Parses a liquid template, returning a Template object.
/// # Examples
///
/// ## Minimal Template
///
/// ```
/// use std::default::Default;
/// use liquid::Renderable;
/// use liquid::LiquidOptions;
/// use liquid::Context;
///
/// let mut options : LiquidOptions = Default::default();
/// let template = liquid::parse("Liquid!", options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Liquid!".to_owned()));
/// ```
///
pub fn parse (text: &str, options: LiquidOptions) -> Result<Template>{
    let mut options = options;
    let tokens = try!(lexer::tokenize(&text));
    options.blocks.insert("raw".to_owned(), Box::new(raw_block));
    options.blocks.insert("if".to_owned(), Box::new(if_block));
    options.blocks.insert("for".to_owned(), Box::new(for_block));
    options.blocks.insert("comment".to_owned(), Box::new(comment_block));

    parser::parse(&tokens, &options).map(|renderables| Template::new(renderables))
}

