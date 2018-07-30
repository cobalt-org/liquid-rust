//! Lexer
//!
//! This module contains elements than can be used for writing plugins
//! but can be ignored for simple usage.

use std::fmt;

use regex::Regex;

use super::{Error, Result};

use super::ComparisonOperator;
use super::Token;

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Expression(Vec<Token>, String),
    Tag(Vec<Token>, String),
    Raw(String),
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Element::Expression(_, ref x) | Element::Tag(_, ref x) | Element::Raw(ref x) => x,
        };
        write!(f, "{}", out)
    }
}
lazy_static! {
    static ref MARKUP: Regex = {
        let t = "(?:[[:space:]]*\\{\\{-|\\{\\{).*?(?:-\\}\\}[[:space:]]*|\\}\\})";
        let e = "(?:[[:space:]]*\\{%-|\\{%).*?(?:-%\\}[[:space:]]*|%\\})";
        Regex::new(&format!("{}|{}", t, e)).unwrap()
    };
}

fn split_blocks(text: &str) -> Vec<&str> {
    let mut tokens = vec![];
    let mut current = 0;
    for mat in MARKUP.find_iter(text) {
        let start = mat.start();
        let end = mat.end();
        match &text[current..start] {
            "" => {}
            t => tokens.push(t),
        }
        tokens.push(&text[start..end]);
        current = end;
    }
    match &text[current..text.len()] {
        "" => {}
        t => tokens.push(t),
    }
    tokens
}

lazy_static! {
    static ref EXPRESSION: Regex = {
        let t = "(?:[[:space:]]*\\{\\{-|\\{\\{)(.*?)(?:-\\}\\}[[:space:]]*|\\}\\})";
        Regex::new(t).unwrap()
    };
    static ref TAG: Regex = {
        let e = "(?:[[:space:]]*\\{%-|\\{%)(.*?)(?:-%\\}[[:space:]]*|%\\})";
        Regex::new(e).unwrap()
    };
}

pub fn tokenize(text: &str) -> Result<Vec<Element>> {
    let mut blocks = vec![];

    for block in split_blocks(text) {
        if let Some(caps) = TAG.captures(block) {
            blocks.push(Element::Tag(
                granularize(caps.get(1).map(|x| x.as_str()).unwrap_or(""))?,
                block.to_owned(),
            ));
        } else if let Some(caps) = EXPRESSION.captures(block) {
            blocks.push(Element::Expression(
                granularize(caps.get(1).map(|x| x.as_str()).unwrap_or(""))?,
                block.to_owned(),
            ));
        } else {
            blocks.push(Element::Raw(block.to_owned()));
        }
    }

    Ok(blocks)
}

lazy_static! {
    static ref SPLIT: Regex =
        Regex::new(r#"'.*?'|".*?"|\s+|[\|:,\[\]\(\)\?]|\.\.|={1,2}|!=|<=|>=|[<>]"#).unwrap();
}

fn split_atom(block: &str) -> Vec<&str> {
    let mut tokens = vec![];
    let mut current = 0;
    for mat in SPLIT.find_iter(block) {
        let start = mat.start();
        let end = mat.end();
        // insert the stuff between identifiers
        tokens.push(&block[current..start]);
        // insert the identifier
        tokens.push(&block[start..end]);
        current = end;
    }
    // insert remaining things
    tokens.push(&block[current..block.len()]);
    tokens
}

lazy_static! {
    static ref IDENTIFIER: Regex = Regex::new(r"[a-zA-Z_][\w-]*\??").unwrap();
    static ref INDEX: Regex = Regex::new(r"^\.[a-zA-Z_][a-zA-Z0-9_-]*").unwrap();
    static ref SINGLE_STRING_LITERAL: Regex = Regex::new(r"'[^']*'").unwrap();
    static ref DOUBLE_STRING_LITERAL: Regex = Regex::new("\"[^\"]*\"").unwrap();
    static ref NUMBER_LITERAL: Regex = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();
    static ref BOOLEAN_LITERAL: Regex = Regex::new(r"^true|false$").unwrap();
}

pub fn granularize(block: &str) -> Result<Vec<Token>> {
    let mut result = vec![];

    let mut push_more;
    for el in split_atom(block) {
        push_more = None;
        result.push(match &*el.trim() {
            "" => continue,

            "|" => Token::Pipe,
            "." => Token::Dot,
            ":" => Token::Colon,
            "," => Token::Comma,
            "[" => Token::OpenSquare,
            "]" => Token::CloseSquare,
            "(" => Token::OpenRound,
            ")" => Token::CloseRound,
            "?" => Token::Question,
            "-" => Token::Dash,
            "=" => Token::Assignment,
            "or" => Token::Or,

            "==" => Token::Comparison(ComparisonOperator::Equals),
            "!=" => Token::Comparison(ComparisonOperator::NotEquals),
            "<=" => Token::Comparison(ComparisonOperator::LessThanEquals),
            ">=" => Token::Comparison(ComparisonOperator::GreaterThanEquals),
            "<" => Token::Comparison(ComparisonOperator::LessThan),
            ">" => Token::Comparison(ComparisonOperator::GreaterThan),
            "contains" => Token::Comparison(ComparisonOperator::Contains),
            ".." => Token::DotDot,

            x if SINGLE_STRING_LITERAL.is_match(x) || DOUBLE_STRING_LITERAL.is_match(x) => {
                Token::StringLiteral(x[1..x.len() - 1].to_owned())
            }
            x if NUMBER_LITERAL.is_match(x) => x.parse::<i32>()
                .map(Token::IntegerLiteral)
                .unwrap_or_else(|_e| {
                    let x = x.parse::<f64>()
                        .expect("matches to NUMBER_LITERAL are parseable as floats");
                    Token::FloatLiteral(x)
                }),
            x if BOOLEAN_LITERAL.is_match(x) => Token::BooleanLiteral(
                x.parse::<bool>()
                    .expect("matches to BOOLEAN_LITERAL are parseable as bools"),
            ),
            x if INDEX.is_match(x) => {
                let mut parts = x.splitn(2, '.');
                parts.next().unwrap();
                push_more = Some(vec![Token::Identifier(parts.next().unwrap().to_owned())]);
                Token::Dot
            }
            x if IDENTIFIER.is_match(x) => Token::Identifier(x.to_owned()),
            x => return Err(Error::with_msg("Invalid identifier").context("identifier", &x)),
        });
        if let Some(v) = push_more {
            result.extend(v);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_blocks() {
        assert_eq!(
            split_blocks("asdlkjfn\n{{askdljfbalkjsdbf}} asdjlfb"),
            vec!["asdlkjfn\n", "{{askdljfbalkjsdbf}}", " asdjlfb"]
        );
        assert_eq!(
            split_blocks("asdlkjfn\n{%askdljfbalkjsdbf%} asdjlfb"),
            vec!["asdlkjfn\n", "{%askdljfbalkjsdbf%}", " asdjlfb"]
        );
    }
    #[test]
    fn test_whitespace_control() {
        assert_eq!(
            split_blocks("foo {{ bar }} 2000"),
            vec!["foo ", "{{ bar }}", " 2000"]
        );
        assert_eq!(
            split_blocks("foo {{- bar -}} 2000"),
            vec!["foo", " {{- bar -}} ", "2000"]
        );
        assert_eq!(
            split_blocks("foo \n{{- bar }} 2000"),
            vec!["foo", " \n{{- bar }}", " 2000"]
        );
        assert_eq!(
            split_blocks("foo {% bar %} 2000"),
            vec!["foo ", "{% bar %}", " 2000"]
        );
        assert_eq!(
            split_blocks("foo {%- bar -%} 2000"),
            vec!["foo", " {%- bar -%} ", "2000"]
        );
        assert_eq!(
            split_blocks("foo \n{%- bar %} 2000"),
            vec!["foo", " \n{%- bar %}", " 2000"]
        );
    }

    #[test]
    fn test_split_atom() {
        assert_eq!(
            split_atom("truc | arg:val"),
            vec!["truc", " ", "", "|", "", " ", "arg", ":", "val"]
        );
        assert_eq!(
            split_atom("truc | filter:arg1,arg2"),
            vec![
                "truc", " ", "", "|", "", " ", "filter", ":", "arg1", ",", "arg2",
            ]
        );
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize("{{hello 'world'}}").unwrap(),
            vec![Element::Expression(
                vec![
                    Token::Identifier("hello".to_owned()),
                    Token::StringLiteral("world".to_owned()),
                ],
                "{{hello 'world'}}".to_owned(),
            )]
        );
        assert_eq!(
            tokenize("{{hello.world}}").unwrap(),
            vec![Element::Expression(
                vec![Token::Identifier("hello.world".to_owned())],
                "{{hello.world}}".to_owned(),
            )]
        );
        assert_eq!(
            tokenize("{{ hello 'world' }}").unwrap(),
            vec![Element::Expression(
                vec![
                    Token::Identifier("hello".to_owned()),
                    Token::StringLiteral("world".to_owned()),
                ],
                "{{ hello 'world' }}".to_owned(),
            )]
        );
        assert_eq!(
            tokenize("{{   hello   'world'    }}").unwrap(),
            vec![Element::Expression(
                vec![
                    Token::Identifier("hello".to_owned()),
                    Token::StringLiteral("world".to_owned()),
                ],
                "{{   hello   'world'    }}".to_owned(),
            )]
        );
        assert_eq!(
            tokenize("wat\n{{hello 'world'}} test").unwrap(),
            vec![
                Element::Raw("wat\n".to_owned()),
                Element::Expression(
                    vec![
                        Token::Identifier("hello".to_owned()),
                        Token::StringLiteral("world".to_owned()),
                    ],
                    "{{hello 'world'}}".to_owned(),
                ),
                Element::Raw(" test".to_owned()),
            ]
        );
        assert_eq!(
            tokenize("wat \n {{-hello 'world'-}} test").unwrap(),
            vec![
                Element::Raw("wat".to_owned()),
                Element::Expression(
                    vec![
                        Token::Identifier("hello".to_owned()),
                        Token::StringLiteral("world".to_owned()),
                    ],
                    " \n {{-hello 'world'-}} ".to_owned(),
                ),
                Element::Raw("test".to_owned()),
            ]
        );
    }

    #[test]
    fn test_granularize() {
        assert_eq!(
            granularize("include my-file.html").unwrap(),
            vec![
                Token::Identifier("include".to_owned()),
                Token::Identifier("my-file.html".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test | me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Pipe,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test .. me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::DotDot,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test : me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Colon,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test , me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comma,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test [ me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::OpenSquare,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test ] me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::CloseSquare,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test ( me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::OpenRound,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test ) me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::CloseRound,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test ? me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Question,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test - me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Dash,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test = me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Assignment,
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test == me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::Equals),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test >= me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::GreaterThanEquals),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test > me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::GreaterThan),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test < me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::LessThan),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test != me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::NotEquals),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test <= me").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Comparison(ComparisonOperator::LessThanEquals),
                Token::Identifier("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test.me").unwrap(),
            vec![Token::Identifier("test.me".to_owned())]
        );
        assert_eq!(
            granularize("'test' == \"me\"").unwrap(),
            vec![
                Token::StringLiteral("test".to_owned()),
                Token::Comparison(ComparisonOperator::Equals),
                Token::StringLiteral("me".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test | me:arg").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Pipe,
                Token::Identifier("me".to_owned()),
                Token::Colon,
                Token::Identifier("arg".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test | me:arg1,arg2").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Pipe,
                Token::Identifier("me".to_owned()),
                Token::Colon,
                Token::Identifier("arg1".to_owned()),
                Token::Comma,
                Token::Identifier("arg2".to_owned()),
            ]
        );
        assert_eq!(
            granularize("test | me : arg1, arg2").unwrap(),
            vec![
                Token::Identifier("test".to_owned()),
                Token::Pipe,
                Token::Identifier("me".to_owned()),
                Token::Colon,
                Token::Identifier("arg1".to_owned()),
                Token::Comma,
                Token::Identifier("arg2".to_owned()),
            ]
        );
        assert_eq!(
            granularize("multiply 5 3").unwrap(),
            vec![
                Token::Identifier("multiply".to_owned()),
                Token::IntegerLiteral(5i32),
                Token::IntegerLiteral(3i32),
            ]
        );
        assert_eq!(
            granularize("multiply 5.5 3.2434").unwrap(),
            vec![
                Token::Identifier("multiply".to_owned()),
                Token::FloatLiteral(5.5f64),
                Token::FloatLiteral(3.2434f64),
            ]
        );
        assert_eq!(
            granularize("for i in (1..5)").unwrap(),
            vec![
                Token::Identifier("for".to_owned()),
                Token::Identifier("i".to_owned()),
                Token::Identifier("in".to_owned()),
                Token::OpenRound,
                Token::IntegerLiteral(1i32),
                Token::DotDot,
                Token::IntegerLiteral(5i32),
                Token::CloseRound,
            ]
        );
        assert_eq!(
            granularize("\"1, '2', 3, 4\"").unwrap(),
            vec![Token::StringLiteral("1, '2', 3, 4".to_owned())]
        );
        assert_eq!(
            granularize("'1, \"2\", 3, 4'").unwrap(),
            vec![Token::StringLiteral("1, \"2\", 3, 4".to_owned())]
        );
        assert_eq!(
            granularize("\"1, '2', 3, 4\"\"1, '2', 3, 4\"").unwrap(),
            vec![
                Token::StringLiteral("1, '2', 3, 4".to_owned()),
                Token::StringLiteral("1, '2', 3, 4".to_owned()),
            ]
        );
        assert_eq!(
            granularize("abc : \"1, '2', 3, 4\"").unwrap(),
            vec![
                Token::Identifier("abc".to_owned()),
                Token::Colon,
                Token::StringLiteral("1, '2', 3, 4".to_owned()),
            ]
        );
    }
}
