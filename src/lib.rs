#![feature(globs)]
#![feature(slicing_syntax)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::collections::HashMap;
use template::Template;
use lexer::Token;

mod template;
mod variable;
mod text;
mod lexer;
mod parser;

pub trait Block {
    fn initialize(&self, tag_name: &str, arguments: &Vec<Token>, tokens: &Vec<Box<Renderable>>) -> Box<Renderable>;
}

pub trait Tag {
    fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: &Vec<Box<Renderable>>) -> Box<Renderable>;
}

pub struct LiquidOptions<'a> {
    blocks : HashMap<String, Box<Block + 'a>>,
    tags : HashMap<String, Box<Tag + 'a>>
}

pub trait Renderable{
    fn render(&self, context: &HashMap<String, String>) -> String;
}

pub fn parse<'a> (text: &str, options: &'a LiquidOptions<'a>) -> Template<'a>{
    let tokens = lexer::tokenize(text.as_slice());
    let renderables = parser::parse(tokens, options);
    Template::new(renderables)
}

#[test]
fn test_liquid() {
    struct Multiply{
        numbers: Vec<int>
    }
    impl Renderable for Multiply{
        fn render(&self, context: &HashMap<String, String>) -> String{
            let x = self.numbers.iter().fold(1, |a, &b| a * b);
            x.to_string()
        }
    }

    struct MultiplyTag;
    impl Tag for MultiplyTag{
        fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: &Vec<Box<Renderable>>) -> Box<Renderable>{
            let numbers = arguments.iter().filter_map( |x| {
                match x {
                    &Token::NumberLiteral(ref num) => from_str(num.as_slice()),
                    _ => None
                }
                }).collect();
            box Multiply{numbers: numbers} as Box<Renderable>
        }
    }

    let mut blocks = HashMap::new();
    let mut tags = HashMap::new();
    tags.insert("multiply".to_string(), box MultiplyTag as Box<Tag>);

    let options = LiquidOptions {
        blocks: blocks,
        tags: tags,
    };
    let template = parse("wat\n{{hello}}\n{{multiply 5 3}} test", &options);

    let mut data = HashMap::new();
    data.insert("hello".to_string(), "world".to_string());

    let output = template.render(&data);
    assert_eq!(output, "wat\nworld\n15 test".to_string());
}

