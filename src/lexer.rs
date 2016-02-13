use self::Token::*;
use self::Element::*;
use self::ComparisonOperator::*;
use regex::Regex;
use error::{Error, Result};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Contains,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Pipe,
    Dot,
    Colon,
    Comma,
    OpenSquare,
    CloseSquare,
    OpenRound,
    CloseRound,
    Question,
    Dash,

    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f32),
    DotDot,
    Comparison(ComparisonOperator),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Pipe => "|".to_owned(),
            Dot => ".".to_owned(),
            Colon => ":".to_owned(),
            Comma => ",".to_owned(),
            OpenSquare => "[".to_owned(),
            CloseSquare => "]".to_owned(),
            OpenRound => "(".to_owned(),
            CloseRound => ")".to_owned(),
            Question => "?".to_owned(),
            Dash => "-".to_owned(),
            DotDot => "..".to_owned(),

            Comparison(Equals) => "==".to_owned(),
            Comparison(NotEquals) => "!=".to_owned(),
            Comparison(LessThanEquals) => "<=".to_owned(),
            Comparison(GreaterThanEquals) => ">=".to_owned(),
            Comparison(LessThan) => "<".to_owned(),
            Comparison(GreaterThan) => ">".to_owned(),
            Comparison(Contains) => "contains".to_owned(),
            Identifier(ref x) => x.clone(),
            StringLiteral(ref x) => x.clone(),
            NumberLiteral(ref x) => x.to_string(),
        };
        write!(f, "{}", out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Expression(Vec<Token>, String),
    Tag(Vec<Token>, String),
    Raw(String),
}

fn split_blocks(text: &str) -> Vec<&str> {
    let markup = Regex::new("\\{%.*?%\\}|\\{\\{.*?\\}\\}").unwrap();
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in markup.find_iter(text) {
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

pub fn tokenize(text: &str) -> Result<Vec<Element>> {
    let expression = Regex::new("\\{\\{(.*?)\\}\\}").unwrap();
    let tag = Regex::new("\\{%(.*?)%\\}").unwrap();

    let mut blocks = vec![];

    for block in split_blocks(text) {
        if let Some(caps) = tag.captures(block) {
            blocks.push(Tag(try!(granularize(caps.at(1).unwrap_or(""))),
                            block.to_string()));
        } else if let Some(caps) = expression.captures(block) {
            blocks.push(Expression(try!(granularize(caps.at(1).unwrap_or(""))),
                                   block.to_string()));
        } else {
            blocks.push(Raw(block.to_string()));
        }
    }

    Ok(blocks)
}

fn split_atom(block: &str) -> Vec<String> {

    let mut vec = vec![];
    let mut buff = String::new();
    for c in block.chars() {
        if c == ' ' {
            if !buff.is_empty() {
                vec.push(buff.clone());
                buff.clear();
            }
        } else if c == ',' || c == ':' {
            if !buff.is_empty() {
                vec.push(buff.clone())
            };
            vec.push(c.to_string());
            buff.clear();
        } else {
            buff.push(c);
        }
    }
    vec.push(buff.clone());
    vec
}

fn granularize(block: &str) -> Result<Vec<Token>> {
    let identifier = Regex::new(r"[a-zA-Z_][\w-]*\??").unwrap();
    let single_string_literal = Regex::new(r"'[^']*'").unwrap();
    let double_string_literal = Regex::new("\"[^\"]*\"").unwrap();
    let number_literal = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();
    let dotdot = Regex::new(r"\.\.").unwrap();

    let mut result = vec![];

    for el in split_atom(block) {
        if el == "" {
            continue;
        }
        result.push(match &*el {
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

            "==" => Comparison(Equals),
            "!=" => Comparison(NotEquals),
            "<=" => Comparison(LessThanEquals),
            ">=" => Comparison(GreaterThanEquals),
            "<" => Comparison(LessThan),
            ">" => Comparison(GreaterThan),
            "contains" => Comparison(Contains),

            x if dotdot.is_match(x) => DotDot,
            x if single_string_literal.is_match(x) => StringLiteral(x[1..x.len() - 1].to_string()),
            x if double_string_literal.is_match(x) => StringLiteral(x[1..x.len() - 1].to_string()),
            x if number_literal.is_match(x) => {
                NumberLiteral(x.parse::<f32>().expect(&format!("Could not parse {:?} as float", x)))
            }
            x if identifier.is_match(x) => Identifier(x.to_string()),
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
               vec!["truc", "|", "arg", ":", "val"]);
    assert_eq!(split_atom("truc | filter:arg1,arg2"),
               vec!["truc", "|", "filter", ":", "arg1", ",", "arg2"]);
}

#[test]
fn test_tokenize() {
    assert_eq!(tokenize("{{hello 'world'}}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_string()),
                                    StringLiteral("world".to_string())],
                               "{{hello 'world'}}".to_string())]);
    assert_eq!(tokenize("{{hello.world}}").unwrap(),
               vec![Expression(vec![Identifier("hello.world".to_string())],
                               "{{hello.world}}".to_string())]);
    assert_eq!(tokenize("{{ hello 'world' }}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_string()),
                                    StringLiteral("world".to_string())],
                               "{{ hello 'world' }}".to_string())]);
    assert_eq!(tokenize("{{   hello   'world'    }}").unwrap(),
               vec![Expression(vec![Identifier("hello".to_string()),
                                    StringLiteral("world".to_string())],
                               "{{   hello   'world'    }}".to_string())]);
    assert_eq!(tokenize("wat\n{{hello 'world'}} test").unwrap(),
               vec![Raw("wat\n".to_string()),
                    Expression(vec![Identifier("hello".to_string()),
                                    StringLiteral("world".to_string())],
                               "{{hello 'world'}}".to_string()),
                    Raw(" test".to_string())]);
}

#[test]
fn test_granularize() {
    assert_eq!(granularize("test me").unwrap(),
               vec![Identifier("test".to_string()), Identifier("me".to_string())]);
    assert_eq!(granularize("test == me").unwrap(),
               vec![Identifier("test".to_string()),
                    Comparison(Equals),
                    Identifier("me".to_string())]);
    assert_eq!(granularize("'test' == \"me\"").unwrap(),
               vec![StringLiteral("test".to_string()),
                    Comparison(Equals),
                    StringLiteral("me".to_string())]);
    assert_eq!(granularize("test | me:arg").unwrap(),
               vec![Identifier("test".to_string()),
                    Pipe,
                    Identifier("me".to_string()),
                    Colon,
                    Identifier("arg".to_string())]);
    assert_eq!(granularize("test | me:arg1,arg2").unwrap(),
               vec![Identifier("test".to_string()),
                    Pipe,
                    Identifier("me".to_string()),
                    Colon,
                    Identifier("arg1".to_string()),
                    Comma,
                    Identifier("arg2".to_string())]);
    assert_eq!(granularize("test | me : arg1, arg2").unwrap(),
               vec![Identifier("test".to_string()),
                    Pipe,
                    Identifier("me".to_string()),
                    Colon,
                    Identifier("arg1".to_string()),
                    Comma,
                    Identifier("arg2".to_string())]);
}
