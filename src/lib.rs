#![feature(globs)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::collections::HashMap;
use template::Template;

mod template;
mod variable;
mod text;
mod lexer;
mod parser;

pub trait Renderable {
    fn render(&self, context: &HashMap<String, String>) -> String;
}

pub fn parse (text: &str) -> Template{
    let tokens = lexer::tokenize(text.as_slice());
    let renderables = parser::parse(tokens);
    Template::new(renderables)
}

#[test]
fn test_liquid() {
    let mut data = HashMap::new();
    data.insert("hello".to_string(), "world".to_string());
    let template = parse("wat\n{{hello}} test");
    let output = template.render(&data);
    assert_eq!(output, "wat\nworld test".to_string());
}

