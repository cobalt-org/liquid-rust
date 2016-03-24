use Renderable;
use LiquidOptions;
use value::Value;
use variable::Variable;
use text::Text;
use std::slice::Iter;
use output::{Output, FilterPrototype, VarOrVal};
use token::Token::{self, Identifier, Colon, Comma, Pipe, StringLiteral, NumberLiteral};
use lexer::Element::{self, Expression, Tag, Raw};
use error::{Error, Result};

pub fn parse(elements: &[Element], options: &LiquidOptions) -> Result<Vec<Box<Renderable>>> {
    let mut ret = vec![];
    let mut iter = elements.iter();
    let mut token = iter.next();
    while token.is_some() {
        match *token.unwrap() {
            Expression(ref tokens, _) => ret.push(try!(parse_expression(tokens, options))),
            Tag(ref tokens, _) => ret.push(try!(parse_tag(&mut iter, tokens, options))),
            Raw(ref x) => ret.push(Box::new(Text::new(&x))),
        }
        token = iter.next();
    }
    Ok(ret)
}

// creates an expression, which wraps everything that gets rendered
fn parse_expression(tokens: &[Token], options: &LiquidOptions) -> Result<Box<Renderable>> {
    match tokens[0] {
        Identifier(ref x) if options.tags.contains_key(&x.to_owned()) => {
            options.tags.get(x).unwrap()(&x, &tokens[1..], options)
        }
        _ => parse_output(tokens),
    }
}

// creates an output, basically a wrapper around values, variables and filters
fn parse_output(tokens: &[Token]) -> Result<Box<Renderable>> {
    let entry = match tokens[0] {
        Identifier(ref x) => VarOrVal::Var(Variable::new(&x)),
        StringLiteral(ref x) => VarOrVal::Val(Value::Str(x.to_owned())),
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
            ref x => {
                return Err(Error::Parser(format!("parse_output: expected an Identifier, got {:?}",
                                                 x)))
            }
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
                &StringLiteral(ref x) => args.push(Value::Str(x.to_owned())),
                &NumberLiteral(x) => args.push(Value::Num(x)),
                ref x => {
                    return Err(Error::Parser(format!("parse_output: {:?} not implemented", x)))
                }
            }
        }

        filters.push(FilterPrototype::new(&name, args));
    }

    Ok(Box::new(Output::new(entry, filters)))
}

// a tag can be either a single-element tag or a block, which can contain other
// elements and is delimited by a closing tag named {{end +
// the_name_of_the_tag}}. Tags do not get rendered, but blocks may contain
// renderable expressions
fn parse_tag(iter: &mut Iter<Element>,
             tokens: &[Token],
             options: &LiquidOptions)
             -> Result<Box<Renderable>> {
    let tag = &tokens[0];
    match *tag {
        // is a tag
        Identifier(ref x) if options.tags.contains_key(x) => {
            options.tags.get(x).unwrap()(&x, &tokens[1..], options)
        }

        // is a block
        Identifier(ref x) if options.blocks.contains_key(x) => {
            // Collect all the inner elements of this block until we find a
            // matching "end<blockname>" tag. Note that there may be nested blocks
            // of the same type (and hence have the same closing delimiter) *inside*
            // the body of the block, which would premauturely stop the element
            // collection early if we did a nesting-unaware search for the
            // closing tag.
            //
            // The whole nesting count machinery below is to ensure we only stop
            // collecting elements when we have an un-nested closing tag.

            let end_tag = Identifier("end".to_owned() + &x);
            let mut children = vec![];
            let mut nesting_depth = 0;
            for t in iter {
                if let &Tag(ref tokens, _) = t {
                    match tokens[0] {
                        ref n if n == tag => {
                            nesting_depth += 1;
                        },
                        ref n if n == &end_tag && nesting_depth > 0 => {
                            nesting_depth -= 1;
                        },
                        ref n if n == &end_tag && nesting_depth == 0 => {
                            break
                        },
                        _ => {}
                    }
                };
                children.push(t.clone())
            }
            options.blocks.get(x).unwrap()(&x, &tokens[1..], children, options)
        }

        ref x => Err(Error::Parser(format!("parse_tag: {:?} not implemented", x))),
    }
}

/// Confirm that the next token in a token stream is what you want it
/// to be. The token iterator is moved to the next token in the stream.
pub fn expect(tokens: &mut Iter<Token>, expected: Token) -> Result<()> {
    match tokens.next() {
        Some(x) if *x == expected => Ok(()),
        x => Error::parser(&expected.to_string(), x)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn expect_rejects_unexpected_token() {
        use super::expect;
        use token::Token::{Pipe, Dot, Colon, Comma};
        let token_vec = vec!(Pipe, Dot, Colon);
        let mut tokens = token_vec.iter();

        assert!(expect(&mut tokens, Pipe).is_ok());
        assert!(expect(&mut tokens, Dot).is_ok());
        assert!(expect(&mut tokens, Comma).is_err());
    }
}
