#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use std::collections::HashMap;

mod lexer;
mod parser;

struct Liquid {
    text: String,
    content: HashMap<String, String>
}

impl Liquid {
    pub fn new(text: &str) -> Liquid {
        Liquid { text: text.to_string(), content: HashMap::new() }
    }
    pub fn parse (content: &HashMap<String, String>) -> String{
        "wtf".to_string()
    }
}

