use Renderable;
use LiquidOptions;
use text::Text;
use variable::Variable;
use lexer::Token;
use lexer::Token::{Identifier};
use lexer::Element;
use lexer::Element::{Output, Tag, Raw};

pub fn parse<'a> (tokens: Vec<Element>, options: &'a LiquidOptions) -> Vec<Box<Renderable + 'a>> {
    tokens.iter().map(|token| {
        match token {
            &Output(ref tokens,_) => parse_token(tokens, options),
            &Tag(_,_) => box Variable::new("tag") as Box<Renderable>,
            &Raw(ref x) => box Text::new(x.as_slice()) as Box<Renderable>
        }
    }).collect()
}

fn parse_token<'a> (tokens: &Vec<Token>, options: &'a LiquidOptions) -> Box<Renderable + 'a> {
    match tokens[0] {
        Identifier(ref x) if options.tags.contains_key(&x.to_string()) => {
            options.tags.get(x).unwrap().initialize(x.as_slice(), tokens.tail(), &vec![])
        },
        Identifier(ref x) => box Variable::new(x.as_slice()) as Box<Renderable>,
         _ => box Variable::new("output") as Box<Renderable>
    }
}
