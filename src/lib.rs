#![crate_name = "liquid"]

#![feature(globs)]
#![feature(slicing_syntax)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate test;

use test::Bencher;
use std::collections::HashMap;
use template::Template;
use lexer::Token;
use lexer::Element;
use tags::IfBlock;
use tags::RawBlock;
use std::string::ToString;
use std::default::Default;

mod template;
mod variable;
mod text;
mod lexer;
mod parser;
mod tags;

pub enum ErrorMode{
    Strict,
    Warn,
    Lax
}

pub enum Value{
    Num(f32),
    Str(String),
    Object(HashMap<String, Value>)
}

impl Default for ErrorMode {
    fn default() -> ErrorMode { ErrorMode::Warn }
}

impl ToString for Value{
    fn to_string(&self) -> String{
        match self{
            &Value::Num(ref x) => x.to_string(),
            &Value::Str(ref x) => x.to_string(),
            _ => "[Object object]".to_string() // TODO
        }
    }
}

pub trait Block {
    fn initialize<'a>(&'a self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &'a LiquidOptions<'a>) -> Box<Renderable>;
}

pub trait Tag {
    fn initialize(&self, tag_name: &str, arguments: &[Token], options : &LiquidOptions) -> Box<Renderable>;
}

#[deriving(Default)]
pub struct LiquidOptions<'a> {
    blocks : HashMap<String, Box<Block + 'a>>,
    tags : HashMap<String, Box<Tag + 'a>>,
    error_mode : ErrorMode
}

pub trait Renderable{
    fn render(&self, context: &HashMap<String, Value>) -> Option<String>;
}

pub fn parse<'a> (text: &str, options: &'a mut LiquidOptions<'a>) -> Template<'a>{
    let tokens = lexer::tokenize(text.as_slice());
    options.blocks.insert("raw".to_string(), box RawBlock as Box<Block>);
    options.blocks.insert("if".to_string(), box IfBlock as Box<Block>);
    let renderables = parser::parse(tokens, options);
    Template::new(renderables)
}

#[bench]
fn simple_parse(b: &mut Bencher) {
    let mut options : LiquidOptions = Default::default();
    let template = parse("{%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}", &mut options);

    let mut data = HashMap::new();
    data.insert("num".to_string(), Value::Num(5f32));
    data.insert("numTwo".to_string(), Value::Num(6f32));

    let output = template.render(&data);
    assert_eq!(output.unwrap(), "wat wot".to_string());

    b.iter(|| template.render(&data));
}

#[bench]
fn custom_output(b: &mut Bencher) {
    struct Multiply{
        numbers: Vec<f32>
    }
    impl Renderable for Multiply{
        fn render(&self, context: &HashMap<String, Value>) -> Option<String>{
            let x = self.numbers.iter().fold(1f32, |a, &b| a * b);
            Some(x.to_string())
        }
    }

    struct MultiplyTag;
    impl Tag for MultiplyTag{
        fn initialize(&self, tag_name: &str, arguments: &[Token], options: &LiquidOptions) -> Box<Renderable>{
            let numbers = arguments.iter().filter_map( |x| {
                match x {
                    &Token::NumberLiteral(ref num) => Some(*num),
                    _ => None
                }
            }).collect();
            box Multiply{numbers: numbers} as Box<Renderable>
        }
    }

    let mut tags = HashMap::new();
    tags.insert("multiply".to_string(), box MultiplyTag as Box<Tag>);

    let mut options = LiquidOptions {
        blocks: Default::default(),
        tags: tags,
        error_mode: Default::default()
    };
    let template = parse("wat\n{{hello}}\n{{multiply 5 3}}{%raw%}{{multiply 5 3}}{%endraw%} test", &mut options);

    let mut data = HashMap::new();
    data.insert("hello".to_string(), Value::Str("world".to_string()));

    let output = template.render(&data);
    assert_eq!(output.unwrap(), "wat\nworld\n15{{multiply 5 3}} test".to_string());

    b.iter(|| template.render(&data));
}

