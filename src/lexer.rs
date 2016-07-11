//! Lexer
//!
//! This module contains elements than can be used for writing plugins
//! but can be ignored for simple usage.

use token::Token;
use token::Token::*;
use token::ComparisonOperator::*;
use self::Element::*;
use regex::Regex;
use error::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Expression(Vec<Token>, String),
    Tag(Vec<Token>, String),
    Raw(String),
}

lazy_static! {
    static ref MARKUP: Regex = Regex::new("\\{%.*?%\\}|\\{\\{.*?\\}\\}").unwrap();
}

fn split_blocks(text: &str) -> Vec<&str> {
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in MARKUP.find_iter(text) {
        match &text[current..begin] {
            "" => {}
            t => tokens.push(t),
        }
        tokens.push(&text[begin..end]);
        current = end;
    }
    match &text[current..text.len()] {
        "" => {}
        t => tokens.push(t),
    }
    tokens
}

lazy_static! {
    static ref EXPRESSION: Regex = Regex::new("\\{\\{(.*?)\\}\\}").unwrap();
    static ref TAG: Regex = Regex::new("\\{%(.*?)%\\}").unwrap();
}

pub fn tokenize(text: &str) -> Result<Vec<Element>> {
    let mut blocks = vec![];

    for block in split_blocks(text) {
        if let Some(caps) = TAG.captures(block) {
            blocks.push(Tag(try!(granularize(caps.at(1).unwrap_or(""))),
                            block.to_owned()));
        } else if let Some(caps) = EXPRESSION.captures(block) {
            blocks.push(Expression(try!(granularize(caps.at(1).unwrap_or(""))),
                                   block.to_owned()));
        } else {
            blocks.push(Raw(block.to_owned()));
        }
    }

    Ok(blocks)
}

lazy_static! {
    static ref SPLIT: Regex = Regex::new(
        r#"'.*?'|".*?"|\s+|[\|:,\[\]\(\)\?]|\.\.|={1,2}|!=|<=|>=|[<>]"#).unwrap();
}

fn split_atom(block: &str) -> Vec<&str> {
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in SPLIT.find_iter(block) {
        // insert the stuff between identifiers
        tokens.push(&block[current..begin]);
        // insert the identifier
        tokens.push(&block[begin..end]);
        current = end;
    }
    // insert remaining things
    tokens.push(&block[current..block.len()]);
    tokens
}

lazy_static! {
    static ref IDENTIFIER: Regex = Regex::new(r"[a-zA-Z_][\w-]*\??").unwrap();
    static ref SINGLE_STRING_LITERAL: Regex = Regex::new(r"'[^']*'").unwrap();
    static ref DOUBLE_STRING_LITERAL: Regex = Regex::new("\"[^\"]*\"").unwrap();
    static ref NUMBER_LITERAL: Regex = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();
    static ref BOOLEAN_LITERAL: Regex = Regex::new(r"^true|false$").unwrap();
}

pub fn granularize(block: &str) -> Result<Vec<Token>> {
    let mut result = vec![];

    for el in split_atom(block) {
        result.push(match &*el.trim() {
            "" => continue,

            "|" => Pipe,
            "." => Dot,
            ":" => Colon,
            "," => Comma,
            "[" => OpenSquare,
            "]" => CloseSquare,
            "(" => OpenRound,
            ")" => CloseRound,
            "?" => Question,
            "-" => Dash,
            "=" => Assignment,
            "or" => Or,

            "==" => Comparison(Equals),
            "!=" => Comparison(NotEquals),
            "<=" => Comparison(LessThanEquals),
            ">=" => Comparison(GreaterThanEquals),
            "<" => Comparison(LessThan),
            ">" => Comparison(GreaterThan),
            "contains" => Comparison(Contains),
            ".." => DotDot,

            x if SINGLE_STRING_LITERAL.is_match(x) || DOUBLE_STRING_LITERAL.is_match(x) => {
                StringLiteral(x[1..x.len() - 1].to_owned())
            }
            x if NUMBER_LITERAL.is_match(x) => {
                NumberLiteral(x.parse::<f32>().expect(&format!("Could not parse {:?} as float", x)))
            }
            x if BOOLEAN_LITERAL.is_match(x) => {
                BooleanLiteral(x.parse::<bool>()
                    .expect(&format!("Could not parse {:?} as bool", x)))
            }
            x if IDENTIFIER.is_match(x) => Identifier(x.to_owned()),
            x => return Err(Error::Lexer(format!("{} is not a valid identifier", x))),
        });
    }

    Ok(result)
}

#[test]
fn test_split_blocks() {
    assert_eq!(split_blocks("asdlkjfn\n{{askdljfbalkjsdbf}} asdjlfb"),
               vec!["asdlkjfn\n", "{{askdljfbalkjsdbf}}", " asdjlfb"]);
    assert_eq!(split_blocks("asdlkjfn\n{%askdljfbalkjsdbf%} asdjlfb"),
               vec!["asdlkjfn\n", "{%askdljfbalkjsdbf%}", " asdjlfb"]);
}

#[test]
fn test_split_atom() {
    assert_eq!(split_atom("truc | arg:val"),
               vec!["truc", " ", "", "|", "", " ", "arg", ":", "val"]);
    assert_eq!(split_atom("truc | filter:arg1,arg2"),
               vec!["truc", " ", "", "|", "", " ", "filter", ":", "arg1", ",", "arg2"]);
}

#[test]
fn test_tokenize() {
    assert_eq!(tokenize("{{hello 'world'}}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_owned()),
                                    StringLiteral("world".to_owned())],
                               "{{hello 'world'}}".to_owned())]);
    assert_eq!(tokenize("{{hello.world}}").unwrap(),
               vec![Expression(vec![Identifier("hello.world".to_owned())],
                               "{{hello.world}}".to_owned())]);
    assert_eq!(tokenize("{{ hello 'world' }}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_owned()),
                                    StringLiteral("world".to_owned())],
                               "{{ hello 'world' }}".to_owned())]);
    assert_eq!(tokenize("{{   hello   'world'    }}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_owned()),
                                    StringLiteral("world".to_owned())],
                               "{{   hello   'world'    }}".to_owned())]);
    assert_eq!(tokenize("wat\n{{hello 'world'}} test").unwrap(),
               vec![Raw("wat\n".to_owned()),
                    Expression(vec![Identifier("hello".to_owned()),
                                    StringLiteral("world".to_owned())],
                               "{{hello 'world'}}".to_owned()),
                    Raw(" test".to_owned())]);
}

#[test]
fn test_granularize() {
    assert_eq!(granularize("include my-file.html").unwrap(),
               vec![Identifier("include".to_owned()), Identifier("my-file.html".to_owned())]);
    assert_eq!(granularize("test | me").unwrap(),
               vec![Identifier("test".to_owned()), Pipe, Identifier("me".to_owned())]);
    assert_eq!(granularize("test .. me").unwrap(),
               vec![Identifier("test".to_owned()), DotDot, Identifier("me".to_owned())]);
    assert_eq!(granularize("test : me").unwrap(),
               vec![Identifier("test".to_owned()), Colon, Identifier("me".to_owned())]);
    assert_eq!(granularize("test , me").unwrap(),
               vec![Identifier("test".to_owned()), Comma, Identifier("me".to_owned())]);
    assert_eq!(granularize("test [ me").unwrap(),
               vec![Identifier("test".to_owned()), OpenSquare, Identifier("me".to_owned())]);
    assert_eq!(granularize("test ] me").unwrap(),
               vec![Identifier("test".to_owned()), CloseSquare, Identifier("me".to_owned())]);
    assert_eq!(granularize("test ( me").unwrap(),
               vec![Identifier("test".to_owned()), OpenRound, Identifier("me".to_owned())]);
    assert_eq!(granularize("test ) me").unwrap(),
               vec![Identifier("test".to_owned()), CloseRound, Identifier("me".to_owned())]);
    assert_eq!(granularize("test ? me").unwrap(),
               vec![Identifier("test".to_owned()), Question, Identifier("me".to_owned())]);
    assert_eq!(granularize("test - me").unwrap(),
               vec![Identifier("test".to_owned()), Dash, Identifier("me".to_owned())]);
    assert_eq!(granularize("test me").unwrap(),
               vec![Identifier("test".to_owned()), Identifier("me".to_owned())]);
    assert_eq!(granularize("test = me").unwrap(),
               vec![Identifier("test".to_owned()), Assignment, Identifier("me".to_owned())]);
    assert_eq!(granularize("test == me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(Equals),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test >= me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(GreaterThanEquals),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test > me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(GreaterThan),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test < me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(LessThan),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test != me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(NotEquals),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test <= me").unwrap(),
               vec![Identifier("test".to_owned()),
                    Comparison(LessThanEquals),
                    Identifier("me".to_owned())]);
    assert_eq!(granularize("test.me").unwrap(),
               vec![Identifier("test.me".to_owned())]);
    assert_eq!(granularize("'test' == \"me\"").unwrap(),
               vec![StringLiteral("test".to_owned()),
                    Comparison(Equals),
                    StringLiteral("me".to_owned())]);
    assert_eq!(granularize("test | me:arg").unwrap(),
               vec![Identifier("test".to_owned()),
                    Pipe,
                    Identifier("me".to_owned()),
                    Colon,
                    Identifier("arg".to_owned())]);
    assert_eq!(granularize("test | me:arg1,arg2").unwrap(),
               vec![Identifier("test".to_owned()),
                    Pipe,
                    Identifier("me".to_owned()),
                    Colon,
                    Identifier("arg1".to_owned()),
                    Comma,
                    Identifier("arg2".to_owned())]);
    assert_eq!(granularize("test | me : arg1, arg2").unwrap(),
               vec![Identifier("test".to_owned()),
                    Pipe,
                    Identifier("me".to_owned()),
                    Colon,
                    Identifier("arg1".to_owned()),
                    Comma,
                    Identifier("arg2".to_owned())]);
    assert_eq!(granularize("multiply 5 3").unwrap(),
               vec![Identifier("multiply".to_owned()), NumberLiteral(5f32), NumberLiteral(3f32)]);
    assert_eq!(granularize("for i in (1..5)").unwrap(),
               vec![Identifier("for".to_owned()),
                    Identifier("i".to_owned()),
                    Identifier("in".to_owned()),
                    OpenRound,
                    NumberLiteral(1f32),
                    DotDot,
                    NumberLiteral(5f32),
                    CloseRound]);
    assert_eq!(granularize("\"1, '2', 3, 4\"").unwrap(),
               vec![StringLiteral("1, '2', 3, 4".to_owned())]);
    assert_eq!(granularize("'1, \"2\", 3, 4'").unwrap(),
               vec![StringLiteral("1, \"2\", 3, 4".to_owned())]);
    assert_eq!(granularize("\"1, '2', 3, 4\"\"1, '2', 3, 4\"").unwrap(),
               vec![StringLiteral("1, '2', 3, 4".to_owned()),
                    StringLiteral("1, '2', 3, 4".to_owned())]);
    assert_eq!(granularize("abc : \"1, '2', 3, 4\"").unwrap(),
               vec![Identifier("abc".to_owned()), Colon, StringLiteral("1, '2', 3, 4".to_owned())]);
}
