#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::collections::HashMap;

mod lexer;
mod parser;

pub struct Template<'a>{
    elements: Vec<Box<Renderable +'a>>
}

struct Variable;

trait Renderable {
    fn render(&self, context: &HashMap<String, String>) -> String;
}

impl Renderable for Variable {
    fn render (&self, context: &HashMap<String, String>) -> String{
        "wtf".to_string()
    }
}

impl<'a> Renderable for Template<'a> {
    fn render (&self, context: &HashMap<String, String>) -> String{
        "wtf".to_string()
    }
}

impl<'a> Template<'a> {
    fn new(elements: Vec<String>) -> Template<'a> {
        let test = Template{elements: vec![]};
        Template{elements: vec![box Variable as Box<Renderable>]}
    }
}

pub fn parse (text: &str) -> Template{
    let tokens = lexer::tokenize(text.as_slice());
    let renderables = parser::parse(tokens);
    Template::new(renderables)
}

#[test]
fn test_liquid() {
    let template = parse("wat\n{{hello 'world'}} test");
    let output = template.render(&HashMap::new());
}

