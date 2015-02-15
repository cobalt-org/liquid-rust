#![crate_name = "liquid"]

#![feature(box_syntax)]
#![feature(unboxed_closures)]
#![feature(slicing_syntax)]
#![feature(plugin)]

#![plugin(regex_macros)]
extern crate regex_macros;
extern crate regex;
extern crate test;

use std::collections::HashMap;
use template::Template;
use lexer::Token;
use lexer::Element;
use tags::IfBlock;
use tags::RawBlock;
use std::string::ToString;
use std::default::Default;
use value::Value;

mod template;
mod output;
mod text;
pub mod lexer;
mod parser;
mod tags;
mod filters;
pub mod value;
mod variable;

#[derive(Copy)]
pub enum ErrorMode{
    Strict,
    Warn,
    Lax
}

impl Default for ErrorMode {
    fn default() -> ErrorMode { ErrorMode::Warn }
}

pub trait Block {
    fn initialize<'a>(&'a self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &'a LiquidOptions<'a>) -> Box<Renderable>;
}

pub trait Tag {
    fn initialize(&self, tag_name: &str, arguments: &[Token], options : &LiquidOptions) -> Box<Renderable>;
}

pub trait Renderable{
    fn render(&self, context: &mut Context) -> Option<String>;
}

#[derive(Default)]
pub struct LiquidOptions<'a> {
    pub blocks : HashMap<String, Box<Block + 'a>>,
    pub tags : HashMap<String, Box<Tag + 'a>>,
    pub error_mode : ErrorMode
}

#[derive(Default)]
pub struct Context<'a>{
    pub values : HashMap<String, Value>,
    pub filters : HashMap<String, Box<Fn(&str) -> String + 'a>>
}

pub fn parse<'a> (text: &str, options: &'a mut LiquidOptions<'a>) -> Template<'a>{
    let tokens = lexer::tokenize(&text[]);
    options.blocks.insert("raw".to_string(), box RawBlock as Box<Block>);
    options.blocks.insert("if".to_string(), box IfBlock as Box<Block>);
    let renderables = parser::parse(tokens, options);
    Template::new(renderables)
}

