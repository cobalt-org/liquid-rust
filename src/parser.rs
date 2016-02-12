use Renderable;
use LiquidOptions;
use value::Value;
use variable::Variable;
use text::Text;
use std::slice::Iter;
use output::Output;
use output::FilterPrototype;
use output::VarOrVal;
use lexer::Token;
use lexer::Token::{Identifier, Colon, Comma, Pipe, StringLiteral, NumberLiteral};
use lexer::Element;
use lexer::Element::{Expression, Tag, Raw};
use error::{Error, Result};

pub fn parse(elements: &[Element],
                 options: &LiquidOptions)
                 -> Result<Vec<Box<Renderable>>> {
    let mut ret = vec![];
    let mut iter = elements.iter();
    let mut token = iter.next();
    while token.is_some() {
        match token.unwrap() {
            &Expression(ref tokens, _) => ret.push(try!(parse_expression(tokens, options))),
            &Tag(ref tokens, _) => ret.push(try!(parse_tag(&mut iter, tokens, options))),
            &Raw(ref x) => ret.push(Box::new(Text::new(&x)) as Box<Renderable>),
        }
        token = iter.next();
    }
    Ok(ret)
}

// creates an expression, which wraps everything that gets rendered
fn parse_expression(tokens: &Vec<Token>,
                        options: &LiquidOptions)
                        -> Result<Box<Renderable>> {
    match tokens[0] {
        Identifier(ref x) if options.tags.contains_key(&x.to_string()) => {
            Ok(options.tags.get(x).unwrap()(&x, &tokens[1..], options))
        }
        _ => parse_output(tokens),
    }
}

// creates an output, basically a wrapper around values, variables and filters
fn parse_output(tokens: &Vec<Token>) -> Result<Box<Renderable>> {
    let entry = match tokens[0] {
        Identifier(ref x) => VarOrVal::Var(Variable::new(&x)),
        StringLiteral(ref x) => VarOrVal::Val(Value::Str(x.to_string())),
        ref x => return Err(Error::Parser(format!("parse_output: {:?} not implemented", x))),
    };

    let mut filters = vec![];
    let mut iter = tokens.iter().peekable();
    iter.next();

    while iter.peek() != None {
        if iter.next().unwrap() != &Pipe {
            return Err(Error::Parser("parse_output: expected a pipe".to_owned()));
        }
        let name = match iter.next() {
            Some(&Identifier(ref name)) => name,
            ref x => return Err(Error::Parser(format!("parse_output: expected an Identifier, got {:?}", x))),
        };
        let mut args = vec![];

        match iter.peek() {
            Some(&&Pipe) | None => {
                filters.push(FilterPrototype::new(&name, args));
                continue;
            }
            _ => (),
        }

        if iter.peek().unwrap() != &&Colon {
            return Err(Error::Parser("parse_output: expected a colon".to_owned()));
        }

        iter.next(); // skip colon

        while iter.peek() != None && iter.peek().unwrap() != &&Pipe {
            match iter.next().unwrap() {
                &Comma => continue, // next argument
                &StringLiteral(ref x) => args.push(Value::Str(x.to_string())),
                &NumberLiteral(x) => args.push(Value::Num(x)),
                ref x => return Err(Error::Parser(format!("parse_output: {:?} not implemented", x))),
            }
        }

        filters.push(FilterPrototype::new(&name, args));
    }

    Ok(Box::new(Output::new(entry, filters)) as Box<Renderable>)
}

// a tag can be either a single-element tag or a block, which can contain other
// elements
// and is delimited by a closing tag named {{end + the_name_of_the_tag}}
// tags do not get rendered, but blocks may contain renderable expressions
fn parse_tag(iter: &mut Iter<Element>,
                 tokens: &Vec<Token>,
                 options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    match tokens[0] {

        // is a tag
        Identifier(ref x) if options.tags.contains_key(x) => {
            Ok(options.tags.get(x).unwrap()(&x, &tokens[1..], options))
        }

        // is a block
        Identifier(ref x) if options.blocks.contains_key(x) => {
            let end_tag = Identifier("end".to_string() + &x);
            let mut children = vec![];
            loop {
                children.push(match iter.next() {
                    Some(&Tag(ref tokens, _)) if tokens[0] == end_tag => break,
                    None => break,
                    Some(t) => t.clone(),
                })
            }
            options.blocks.get(x).unwrap()(&x, &tokens[1..], children, options)
        }

        ref x => Err(Error::Parser(format!("parse_tag: {:?} not implemented", x))),
    }
}
