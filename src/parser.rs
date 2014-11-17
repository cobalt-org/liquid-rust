use Renderable;
use text::Text;
use variable::Variable;
use lexer::Element;
use lexer::Token;
use lexer::Identifier;
use lexer::{Output, Tag, Raw};

pub fn parse<'a> (tokens: Vec<Element>) -> Vec<Box<Renderable + 'a>> {
    tokens.iter().map(|token| {
        match token {
            &Output(ref tokens,_) => parse_token(tokens),
            &Tag(_,_) => box Variable::new("tag") as Box<Renderable>,
            &Raw(ref x) => box Text::new(x.as_slice()) as Box<Renderable>
        }
    }).collect()
}

fn parse_token<'a> (tokens: &Vec<Token>) -> Box<Renderable + 'a> {
    let ret = match tokens[0] {
        Identifier(ref x) => box Variable::new(x.as_slice()),
         _ => box Variable::new("output")
    };
    ret as Box<Renderable>
}
