use Renderable;
use LiquidOptions;
use value::Value;
use variable::Variable;
use text::Text;
use output::{Output, FilterPrototype, VarOrVal};
use token::Token::{self, Identifier, Colon, Comma, Pipe, StringLiteral, NumberLiteral};
use lexer::Element::{self, Expression, Tag, Raw};
use error::{Error, Result};

use std::slice::Iter;
use std::collections::HashSet;
use std::iter::FromIterator;

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

/// Extracts a token from the token stream that can be used to express a
/// value. For our purposes, this is either a string literal, number literal
/// or an identifier that might refer to a variable.
pub fn consume_value_token(tokens: &mut Iter<Token>) -> Result<Token> {
    match tokens.next() {
        Some(t) => value_token(t.clone()),
        None => Error::parser("string | number | identifier", None)
    }
}

/// Recognises a value token, returning an error if a non-value token
/// is presented.
pub fn value_token(t: Token) -> Result<Token> {
    match t {
        v @ StringLiteral(_) |
        v @ NumberLiteral(_) |
        v @ Identifier(_) => {
            Ok(v)
        },
        x => {
            Error::parser("string | number | identifier", Some(&x))
        }
    }
}

/// Describes the optional trailing part of a block split.
pub struct BlockSplit<'a> {
    pub delimiter: String,
    pub args: &'a [Token],
    pub trailing: &'a [Element]
}

/// A sub-block aware splitter that will only split the token stream
/// when it finds a delimter at the top level of the token stream,
/// ignoring any it finds in nested blocks.
///
/// Returns a slice contaiing all elements before the delimiter, and
/// an optional BlockSplit struct describing the delimiter and
/// trailing elements.
pub fn split_block<'a>(tokens: &'a[Element],
                       delimiters: &[&str],
                       options: &LiquidOptions) ->
                            (&'a[Element], Option<BlockSplit<'a>>) {
    // construct a fast-lookup cache of the delimiters, as we're going to be
    // consulting the delimiter list a *lot*.
    let delims : HashSet<&str> = HashSet::from_iter(delimiters.iter().map(|x|*x));
    let mut stack : Vec<String> = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        if let Tag(ref args, _) = *t {
            match args[0] {
                Identifier(ref name) if options.blocks.contains_key(name) => {
                    stack.push("end".to_owned() + name);
                },

                Identifier(ref name) if Some(name) == stack.last() => {
                    stack.pop();
                },

                Identifier(ref name) if stack.is_empty() &&
                                        delims.contains(name.as_str()) => {
                    let leading = &tokens[0..i];
                    let split = BlockSplit {
                        delimiter: name.clone(),
                        args: args,
                        trailing: &tokens[i..]
                    };
                    return (leading, Some(split));
                },
                _ => {}
            }
        }
    }

    (&tokens[..], None)
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

    #[test]
    fn token_split_handles_nonmatching_stream() {
        use lexer::tokenize;
        use super::split_block;
        use LiquidOptions;

        // A stream of tokens with lots of `else`s in it, but only one at the
        // top level, which is where it should split.
        let tokens = tokenize(
            "{% comment %}A{%endcomment%} bunch of {{text}} with {{no}} else tag"
        ).unwrap();

        // note that we need an options block that has been initilaised with
        // the supported block list; otherwise the split_tag function won't know
        // which things start a nested block.
        let options = LiquidOptions::with_known_blocks();
        let (_, trailing) = split_block(&tokens[..], &["else"], &options);
        assert!(trailing.is_none());
    }


    #[test]
    fn token_split_honours_nesting() {
        use lexer::tokenize;
        use token::Token::Identifier;
        use lexer::Element::{Tag, Raw};
        use super::split_block;
        use LiquidOptions;

        // A stream of tokens with lots of `else`s in it, but only one at the
        // top level, which is where it should split.
        let tokens = tokenize(concat!(
            "{% for x in (1..10) %}",
                "{% if x == 2 %}",
                    "{% for y (2..10) %}{{y}}{% else %} zz {% endfor %}",
                "{% else %}",
                    "c",
                "{% endif %}",
            "{% else %}",
                "something",
            "{% endfor %}",
            "{% else %}",
            "trailing tags"
            )).unwrap();

        // note that we need an options block that has been initilaised with
        // the supported block list; otherwise the split_tag function won't know
        // which things start a nested block.
        let options = LiquidOptions::with_known_blocks();
        let (_, trailing) = split_block(&tokens[..], &["else"], &options);
        match trailing {
            Some(split) => {
                assert_eq!(split.delimiter, "else");
                assert_eq!(split.args, &[Identifier("else".to_owned())]);
                assert_eq!(split.trailing, &[
                    Tag(vec![Identifier("else".to_owned())], "{% else %}".to_owned()),
                    Raw("trailing tags".to_owned())]);
            },
            None => panic!("split failed")
        }
    }
}