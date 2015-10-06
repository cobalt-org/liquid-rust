#![crate_name = "liquid"]
#![doc(html_root_url = "https://cobalt-org.github.io/liquid-rust/")]

extern crate regex;

use std::collections::HashMap;
use lexer::Token;
use lexer::Element;
use tags::{IfBlock, ForBlock, RawBlock, CommentBlock};
use std::string::ToString;
use std::default::Default;
pub use value::Value;
pub use context::Context;
pub use template::Template;

mod template;
mod output;
mod text;
pub mod lexer;
mod parser;
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

/// A trait for creating custom tags.
pub trait Tag {
    fn initialize(&self, tag_name: &str, arguments: &[Token], options : &LiquidOptions) -> Box<Renderable>;
}

/// The trait to use when implementing custom block-size tags ({% if something %})
pub trait Block {
    fn initialize<'a>(&'a self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &'a LiquidOptions<'a>) -> Result<Box<Renderable +'a>, String>;
}

/// Any object (tag/block) that can be rendered by liquid must implement this trait.
pub trait Renderable{
    fn render(&self, context: &mut Context) -> Option<String>;
}

#[derive(Default)]
pub struct LiquidOptions<'a> {
    pub blocks : HashMap<String, Box<Block + 'a>>,
    pub tags : HashMap<String, Box<Tag + 'a>>,
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
/// let mut options : LiquidOptions = Default::default();
/// let template = liquid::parse("Liquid!", &mut options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), "Liquid!".to_string());
/// ```
///
pub fn parse<'a, 'b> (text: &str, options: &'b mut LiquidOptions<'a>) -> Result<Template<'b>, String>{
    let tokens = lexer::tokenize(&text);
    options.blocks.insert("raw".to_string(), Box::new(RawBlock) as Box<Block>);
    options.blocks.insert("if".to_string(), Box::new(IfBlock) as Box<Block>);
    options.blocks.insert("for".to_string(), Box::new(ForBlock) as Box<Block>);
    options.blocks.insert("comment".to_string(), Box::new(CommentBlock) as Box<Block>);
    match parser::parse(&tokens, options) {
        Ok(renderables) => Ok(Template::new(renderables)),
        Err(e) => Err(e)
    }
}

