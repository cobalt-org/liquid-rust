use self::Token::*;
use self::Element::*;
use self::ComparisonOperator::*;
use regex::Regex;

static MARKUP     : Regex = regex!("\\{%.*?%\\}|\\{\\{.*?\\}\\}");
static TAG        : Regex = regex!("\\{%(.*?)%\\}");
static EXPRESSION : Regex = regex!("\\{\\{(.*?)\\}\\}");
static WHITESPACE : Regex = regex!(r"\s+");

static IDENTIFIER            : Regex = regex!(r"[a-zA-Z_][\w-]*\??");
static SINGLE_STRING_LITERAL : Regex = regex!(r"'[^']*'");
static DOUBLE_STRING_LITERAL : Regex = regex!("\"[^\"]*\"");
static NUMBER_LITERAL        : Regex = regex!(r"-?\d+(\.\d+)?");
static DOTDOT                : Regex = regex!(r"\.\.");

#[derive(Clone, Show, PartialEq)]
pub enum ComparisonOperator{
    Equals, NotEquals,
    LessThan, GreaterThan,
    LessThanEquals, GreaterThanEquals,
    Contains
}

#[derive(Clone, Show, PartialEq)]
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
    Comparison(ComparisonOperator)
}

#[derive(Clone, Show, PartialEq)]
pub enum Element{
    Expression(Vec<Token>, String),
    Tag(Vec<Token>, String),
    Raw(String)
}

fn split_blocks(text: &str) -> Vec<&str>{
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in MARKUP.find_iter(text) {
        match &text[current..begin]{
            "" => {}
            t => tokens.push(t)
        };
        tokens.push(&text[begin..end]);
        current = end;
    }
    match &text[current..text.len()]{
        "" => {}
        t => tokens.push(t)
    };
    tokens
}

pub fn tokenize(text: &str) -> Vec<Element> {
    split_blocks(text).iter().map(|block| {
        if TAG.is_match(*block) {
            let caps = TAG.captures(*block).unwrap();
            Tag(granularize(caps.at(1).unwrap()), block.to_string())
        }else if EXPRESSION.is_match(*block) {
            let caps = EXPRESSION.captures(*block).unwrap();
            Expression(granularize(caps.at(1).unwrap()), block.to_string())
        }else{
            Raw(block.to_string())
        }
    }).collect()
}

fn granularize(block: &str) -> Vec<Token>{
    WHITESPACE.split(block).filter_map(|el|{
        if el == ""{
            return None;
        }
        Some(match el {
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

            "=="       => Comparison(Equals),
            "!="       => Comparison(NotEquals),
            "<="       => Comparison(LessThanEquals),
            ">="       => Comparison(GreaterThanEquals),
            "<"        => Comparison(LessThan),
            ">"        => Comparison(GreaterThan),
            "contains" => Comparison(Contains),

            x if DOTDOT.is_match(x) => DotDot,
            x if SINGLE_STRING_LITERAL.is_match(x) => StringLiteral(x[1..x.len() -1].to_string()),
            x if DOUBLE_STRING_LITERAL.is_match(x) => StringLiteral(x[1..x.len() -1].to_string()),
            x if NUMBER_LITERAL.is_match(x) => NumberLiteral(x.parse::<f32>().unwrap()),
            x if IDENTIFIER.is_match(x) => Identifier(x.to_string()),
            x => panic!("{} is not a valid identifier", x)
        })
    }).collect()
}

#[test]
fn test_split_blocks() {
    assert_eq!(split_blocks("asdlkjfn\n{{askdljfbalkjsdbf}} asdjlfb"),
                vec!["asdlkjfn\n", "{{askdljfbalkjsdbf}}", " asdjlfb"]);
    assert_eq!(split_blocks("asdlkjfn\n{%askdljfbalkjsdbf%} asdjlfb"),
                vec!["asdlkjfn\n", "{%askdljfbalkjsdbf%}", " asdjlfb"]);
}

#[test]
fn test_tokenize() {
    assert_eq!(tokenize("{{hello 'world'}}"), vec![
               Expression(vec![Identifier("hello".to_string()), StringLiteral("world".to_string())], "{{hello 'world'}}".to_string())
               ]);
    assert_eq!(tokenize("{{ hello 'world' }}"), vec![
               Expression(vec![Identifier("hello".to_string()), StringLiteral("world".to_string())], "{{ hello 'world' }}".to_string())
               ]);
    assert_eq!(tokenize("{{   hello   'world'    }}"), vec![
               Expression(vec![Identifier("hello".to_string()), StringLiteral("world".to_string())], "{{   hello   'world'    }}".to_string())
               ]);
    assert_eq!(tokenize("wat\n{{hello 'world'}} test"), vec![
               Raw("wat\n".to_string()), Expression(vec![Identifier("hello".to_string()), StringLiteral("world".to_string())], "{{hello 'world'}}".to_string()), Raw(" test".to_string())
               ]);
}

#[test]
fn test_granularize() {
    assert_eq!(granularize("test me"), vec![Identifier("test".to_string()), Identifier("me".to_string())]);
    assert_eq!(granularize("test == me"), vec![Identifier("test".to_string()), Comparison(Equals), Identifier("me".to_string())]);
    assert_eq!(granularize("'test' == \"me\""), vec![StringLiteral("test".to_string()), Comparison(Equals), StringLiteral("me".to_string())]);
}

