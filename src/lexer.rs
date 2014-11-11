use regex::Regex;

static TAGS : Regex = regex!("\\{%.*?%\\}|\\{\\{.*?\\}\\}");
static EXPRESSION : Regex = regex!("\\{%.*?%\\}");
static STATEMENT : Regex = regex!("\\{\\{.*?\\}\\}");
static WHITESPACE : Regex = regex!(r"\s+");

#[deriving(Show, PartialEq)]
enum Token {
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

    Identifier,
    SingleStringLiteral,
    DoubleStringLiteral,
    NumberLiteral,
    DotDot,
    ComparisonOperator
}

#[deriving(Show, PartialEq)]
enum Element{
    Statement(Vec<Token>, String),
    Expression(Vec<Token>, String),
    Raw(String)
}

fn split_blocks(text: &str) -> Vec<&str>{
    let mut tokens = vec![];
    let mut current = 0;
    for (begin, end) in TAGS.find_iter(text) {
        match text.slice(current, begin){
            "" => {}
            t => tokens.push(t)
        };
        tokens.push(text.slice(begin, end));
        current = end;
    }
    match text.slice(current, text.len()){
        "" => {}
        t => tokens.push(t)
    };
    tokens
}

fn tokenize(text: &str) -> Vec<Element> {
    let blocks = split_blocks(text);
    blocks.iter().map(|block| {
        if(EXPRESSION.is_match(*block)){
            Expression(granularize(*block), block.to_string())
        }else if(STATEMENT.is_match(*block)){
            Statement(granularize(*block), block.to_string())
        }else{
            Raw(block.to_string())
        }
    }).collect()
}

fn granularize(block: &str) -> Vec<Token>{
    vec!()
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
    assert_eq!(tokenize("asdlkjfn\n{{askdljfbalkjsdbf}} asdjlfb"), vec![
               Raw("asdlkjfn\n".to_string()), Statement(vec!(), "{{askdljfbalkjsdbf}}".to_string()), Raw(" asdjlfb".to_string())
               ]);
}

