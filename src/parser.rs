use Renderable;
use LiquidOptions;
use text::Text;
use std::slice::Items;
use variable::Variable;
use lexer::Token;
use lexer::Token::{Identifier};
use lexer::Element;
use lexer::Element::{Output, Tag, Raw};

pub fn parse<'a> (elements: Vec<Element>, options: &'a LiquidOptions) -> Vec<Box<Renderable + 'a>> {
    let mut ret = vec![];
    let mut iter = elements.iter();
    let mut token = iter.next();
    while token.is_some() {
        match token.unwrap() {
            &Output(ref tokens,_) => ret.push(parse_output(tokens, options)),
            &Tag(ref tokens,_) => ret.push(parse_tag(&mut iter, tokens, options)),
            &Raw(ref x) => ret.push(box Text::new(x.as_slice()) as Box<Renderable>)
        }
        token = iter.next();
    }
    ret
}

fn parse_output<'a> (tokens: &Vec<Token>, options: &'a LiquidOptions) -> Box<Renderable + 'a> {
    match tokens[0] {
        Identifier(ref x) if options.tags.contains_key(&x.to_string()) => {
            options.tags.get(x).unwrap().initialize(x.as_slice(), tokens.tail())
        },
        Identifier(ref x) => box Variable::new(x.as_slice()) as Box<Renderable>,
        ref x => panic!("{} not implemented", x)
    }
}

fn parse_tag<'a> (iter: &mut Items<Element>, tokens: &Vec<Token>, options: &'a LiquidOptions) -> Box<Renderable + 'a> {
    match tokens[0] {

        // is a tag
        Identifier(ref x) if options.tags.contains_key(&x.to_string()) => {
            options.tags.get(x).unwrap().initialize(x.as_slice(), tokens.tail())
        },

        // is a block
        Identifier(ref x) if options.blocks.contains_key(&x.to_string()) => {
            // TODO this is so ugly
            // Please make this look better
            // it just works
            let end_tag = Identifier("end".to_string() + *x);
            let mut children = vec![];
            loop {
                children.push(match iter.next() {
                    Some(&Tag(ref tokens,_)) if tokens[0] == end_tag => break,
                    None => break,
                    Some(&Output(_, ref t)) => t,
                    Some(&Tag(_, ref t)) => t,
                    Some(&Raw(ref t)) => t,
                })
            }
            options.blocks.get(x).unwrap().initialize(x.as_slice(),
                                                      tokens.tail(),
                                                      // TODO i'm gonna puke
                                                      children.iter().fold("".to_string(), |a, b| a + b.to_string()))
        },
        Identifier(ref x) => box Variable::new(x.as_slice()) as Box<Renderable>,
        ref x => panic!("{} not implemented", x)
    }
}
