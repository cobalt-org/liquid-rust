//! Parser
//!
//! This module contains functions than can be used for writing plugins
//! but should be ignored for simple usage.

use std::slice::Iter;
use std::collections::HashSet;
use std::iter::FromIterator;

use error::{Error, Result};

use interpreter::Renderable;
use interpreter::Text;
use interpreter::Variable;
use interpreter::{Output, FilterPrototype};
use super::Element;
use super::LiquidOptions;
use super::ParseBlock;
use super::ParseTag;
use super::Token;
use value::Index;

/// Parses the provided elements into a number of Renderable items
/// This is the internal version of parse that accepts Elements tokenized
/// by `lexer::tokenize` and does not register built-in blocks. The main use
/// for this function is for writing custom blocks.
///
/// For parsing from a String you should refer to `liquid::parse`.
pub fn parse(elements: &[Element], options: &LiquidOptions) -> Result<Vec<Box<Renderable>>> {
    let mut ret = vec![];
    let mut iter = elements.iter();
    let mut token = iter.next();
    while token.is_some() {
        let render = match *token.unwrap() {
            Element::Expression(ref tokens, _) => parse_expression(tokens, options)?,
            Element::Tag(ref tokens, _) => parse_tag(&mut iter, tokens, options)?,
            Element::Raw(ref x) => Box::new(Text::new(x)),
        };
        ret.push(render);
        token = iter.next();
    }
    Ok(ret)
}

// creates an expression, which wraps everything that gets rendered
fn parse_expression(tokens: &[Token], options: &LiquidOptions) -> Result<Box<Renderable>> {
    match tokens.get(0) {
        Some(&Token::Identifier(ref x)) if tokens.len() > 1 &&
                                           (tokens[1] == Token::Dot ||
                                            tokens[1] == Token::OpenSquare) => {
            let indexes = parse_indexes(&tokens[1..])?;
            let mut result = Variable::new(x.clone());
            result.extend(indexes);
            Ok(Box::new(result))
        }
        Some(&Token::Identifier(ref x)) if options.tags.contains_key(x) => {
            options.tags[x].parse(x, &tokens[1..], options)
        }
        None => Error::parser("expression", None),
        _ => {
            let output = parse_output(tokens)?;
            Ok(Box::new(output))
        }
    }
}

pub fn parse_indexes(mut tokens: &[Token]) -> Result<Vec<Index>> {
    let mut indexes: Vec<Index> = Vec::new();

    let mut rest = 0;
    while tokens.len() > rest {
        tokens = &tokens[rest..];
        rest = match tokens[0] {
            Token::Dot if tokens.len() > 1 => {
                match tokens[1] {
                    Token::Identifier(ref x) => indexes.push(Index::with_key(x.as_ref())),
                    _ => {
                        return Error::parser("identifier", Some(&tokens[0]));
                    }
                };
                2
            }
            Token::OpenSquare if tokens.len() > 2 => {
                let index = match tokens[1] {
                    Token::StringLiteral(ref x) => Index::with_key(x.as_ref()),
                    Token::IntegerLiteral(ref x) => Index::with_index(*x as isize),
                    _ => {
                        return Error::parser("integer | string", Some(&tokens[0]));
                    }
                };
                indexes.push(index);

                if tokens[2] != Token::CloseSquare {
                    return Error::parser("]", Some(&tokens[1]));
                }
                3
            }
            _ => return Ok(indexes),
        };
    }

    Ok(indexes)
}

/// Creates an Output, a wrapper around values, variables and filters
/// used internally, from a list of Tokens. This is mostly useful
/// for correctly parsing complex expressions with filters.
pub fn parse_output(tokens: &[Token]) -> Result<Output> {
    let entry = tokens[0].to_arg()?;

    let mut filters = vec![];
    let mut iter = tokens.iter().peekable();
    iter.next();

    while iter.peek() != None {
        expect(&mut iter, &Token::Pipe)?;

        let name = match iter.next() {
            Some(&Token::Identifier(ref name)) => name,
            x => {
                return Error::parser("an identifier", x);
            }
        };
        let mut args = vec![];

        match iter.peek() {
            Some(&&Token::Pipe) |
            None => {
                filters.push(FilterPrototype::new(name, args));
                continue;
            }
            _ => (),
        }

        expect(&mut iter, &Token::Colon)?;

        // loops through the argument list after the filter name
        while iter.peek() != None && iter.peek().unwrap() != &&Token::Pipe {
            args.push(iter.next().unwrap().to_arg()?);

            // ensure that the next token is either a Comma or a Pipe
            match iter.peek() {
                Some(&&Token::Comma) => {
                    let _ = iter.next().unwrap();
                    continue;
                }
                Some(&&Token::Pipe) |
                None => break,
                _ => {
                    return Error::parser("a comma or a pipe", Some(iter.next().unwrap()));
                }
            }
        }

        filters.push(FilterPrototype::new(name, args));
    }

    Ok(Output::new(entry, filters))
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
        Token::Identifier(ref x) if options.tags.contains_key(x) => {
            options.tags[x].parse(x, &tokens[1..], options)
        }

        // is a block
        Token::Identifier(ref x) if options.blocks.contains_key(x) => {
            // Collect all the inner elements of this block until we find a
            // matching "end<blockname>" tag. Note that there may be nested blocks
            // of the same type (and hence have the same closing delimiter) *inside*
            // the body of the block, which would premauturely stop the element
            // collection early if we did a nesting-unaware search for the
            // closing tag.
            //
            // The whole nesting count machinery below is to ensure we only stop
            // collecting elements when we have an un-nested closing tag.

            let end_tag = Token::Identifier("end".to_owned() + x);
            let mut children = vec![];
            let mut nesting_depth = 0;
            for t in iter {
                if let Element::Tag(ref tokens, _) = *t {
                    match tokens[0] {
                        ref n if n == tag => {
                            nesting_depth += 1;
                        }
                        ref n if n == &end_tag && nesting_depth > 0 => {
                            nesting_depth -= 1;
                        }
                        ref n if n == &end_tag && nesting_depth == 0 => break,
                        _ => {}
                    }
                };
                children.push(t.clone())
            }
            options.blocks[x].parse(x, &tokens[1..], &children, options)
        }

        ref x => Err(Error::Parser(format!("parse_tag: {:?} not implemented", x))),
    }
}

/// Confirm that the next token in a token stream is what you want it
/// to be. The token iterator is moved to the next token in the stream.
pub fn expect<'a, T>(tokens: &mut T, expected: &Token) -> Result<&'a Token>
    where T: Iterator<Item = &'a Token>
{
    match tokens.next() {
        Some(x) if x == expected => Ok(x),
        x => Error::parser(&expected.to_string(), x),
    }
}

/// Extracts a token from the token stream that can be used to express a
/// value. For our purposes, this is either a string literal, number literal
/// or an identifier that might refer to a variable.
pub fn consume_value_token(tokens: &mut Iter<Token>) -> Result<Token> {
    match tokens.next() {
        Some(t) => value_token(t.clone()),
        None => Error::parser("string | number | boolean | identifier", None),
    }
}

/// Recognises a value token, returning an error if a non-value token
/// is presented.
pub fn value_token(t: Token) -> Result<Token> {
    match t {
        v @ Token::StringLiteral(_) |
        v @ Token::IntegerLiteral(_) |
        v @ Token::FloatLiteral(_) |
        v @ Token::BooleanLiteral(_) |
        v @ Token::Identifier(_) => Ok(v),
        x => Error::parser("string | number | boolean | identifier", Some(&x)),
    }
}

/// Describes the optional trailing part of a block split.
pub struct BlockSplit<'a> {
    pub delimiter: String,
    pub args: &'a [Token],
    pub trailing: &'a [Element],
}

/// A sub-block aware splitter that will only split the token stream
/// when it finds a delimter at the top level of the token stream,
/// ignoring any it finds in nested blocks.
///
/// Returns a slice contaiing all elements before the delimiter, and
/// an optional `BlockSplit` struct describing the delimiter and
/// trailing elements.
pub fn split_block<'a>(tokens: &'a [Element],
                       delimiters: &[&str],
                       options: &LiquidOptions)
                       -> (&'a [Element], Option<BlockSplit<'a>>) {
    // construct a fast-lookup cache of the delimiters, as we're going to be
    // consulting the delimiter list a *lot*.
    let delims: HashSet<&str> = HashSet::from_iter(delimiters.iter().map(|x| *x));
    let mut stack: Vec<String> = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        if let Element::Tag(ref args, _) = *t {
            match args[0] {
                Token::Identifier(ref name) if options.blocks.contains_key(name) => {
                    stack.push("end".to_owned() + name);
                }

                Token::Identifier(ref name) if Some(name) == stack.last() => {
                    stack.pop();
                }

                Token::Identifier(ref name) if stack.is_empty() &&
                                               delims.contains(name.as_str()) => {
                    let leading = &tokens[0..i];
                    let split = BlockSplit {
                        delimiter: name.clone(),
                        args: args,
                        trailing: &tokens[i..],
                    };
                    return (leading, Some(split));
                }
                _ => {}
            }
        }
    }

    (&tokens[..], None)
}

#[cfg(test)]
mod test_parse_output {
    use super::*;
    use value::Value;
    use super::super::lexer::granularize;
    use interpreter::Argument;

    #[test]
    fn parses_filters() {
        let tokens = granularize("abc | def:'1',2,'3' | blabla").unwrap();

        let result = parse_output(&tokens);
        assert_eq!(result.unwrap(),
                   Output::new(Argument::Var(Variable::new("abc")),
                               vec![FilterPrototype::new("def",
                                                         vec![Argument::Val(Value::scalar("1")),
                                                              Argument::Val(Value::scalar(2.0)),
                                                              Argument::Val(Value::scalar("3"))]),
                                    FilterPrototype::new("blabla", vec![])]));
    }

    #[test]
    fn requires_filter_names() {
        let tokens = granularize("abc | '1','2','3' | blabla").unwrap();

        let result = parse_output(&tokens);
        assert_eq!(result.unwrap_err().to_string(),
                   "Parsing error: Expected an identifier, found 1");
    }

    #[test]
    fn fails_on_missing_pipes() {
        let tokens = granularize("abc | def:'1',2,'3' blabla").unwrap();

        let result = parse_output(&tokens);
        assert_eq!(result.unwrap_err().to_string(),
                   "Parsing error: Expected a comma or a pipe, found blabla");
    }

    #[test]
    fn fails_on_missing_colons() {
        let tokens = granularize("abc | def '1',2,'3' | blabla").unwrap();

        let result = parse_output(&tokens);
        assert_eq!(result.unwrap_err().to_string(),
                   "Parsing error: Expected :, found 1");
    }
}

#[cfg(test)]
mod test_expect {
    use super::*;

    #[test]
    fn rejects_unexpected_token() {
        let token_vec = vec![Token::Pipe, Token::Dot, Token::Colon];
        let mut tokens = token_vec.iter();

        assert!(expect(&mut tokens, &Token::Pipe).is_ok());
        assert!(expect(&mut tokens, &Token::Dot).is_ok());
        assert!(expect(&mut tokens, &Token::Comma).is_err());
    }
}

#[cfg(test)]
mod test_split_block {
    use std::collections::HashMap;
    use super::*;
    use super::super::tokenize;
    use super::super::split_block;
    use super::super::BoxedBlockParser;
    use super::super::FnParseBlock;
    use interpreter::Renderable;
    use interpreter::Context;
    use interpreter;

    #[derive(Debug)]
    struct NullBlock;

    impl Renderable for NullBlock {
        fn render(&self, _context: &mut Context) -> Result<Option<String>> {
            Ok(None)
        }
    }

    pub fn null_block(_tag_name: &str,
                      _arguments: &[Token],
                      _tokens: &[Element],
                      _options: &LiquidOptions)
                      -> Result<Box<Renderable>> {
        Ok(Box::new(NullBlock))
    }

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        let blocks: HashMap<String, BoxedBlockParser> = ["comment", "for", "if"]
            .into_iter()
            .map(|name| (name.to_string(), (null_block as FnParseBlock).into()))
            .collect();
        options.blocks = blocks;
        options
    }

    #[test]
    fn parse_empty_expression() {
        let text = "{{}}";

        let tokens = tokenize(&text).unwrap();
        let template = parse(&tokens, &options()).map(interpreter::Template::new);
        assert!(template.is_err());
    }

    #[test]
    fn handles_nonmatching_stream() {
        // A stream of tokens with lots of `else`s in it, but only one at the
        // top level, which is where it should split.
        let tokens = tokenize("{% comment %}A{%endcomment%} bunch of {{text}} with {{no}} \
                                   else tag")
            .unwrap();

        // note that we need an options block that has been initilaised with
        // the supported block list; otherwise the split_tag function won't know
        // which things start a nested block.
        let options = options();
        let (_, trailing) = split_block(&tokens[..], &["else"], &options);
        assert!(trailing.is_none());
    }

    #[test]
    fn honours_nesting() {
        // A stream of tokens with lots of `else`s in it, but only one at the
        // top level, which is where it should split.
        let tokens = tokenize(concat!("{% for x in (1..10) %}",
                                      "{% if x == 2 %}",
                                      "{% for y (2..10) %}{{y}}{% else %} zz {% endfor %}",
                                      "{% else %}",
                                      "c",
                                      "{% endif %}",
                                      "{% else %}",
                                      "something",
                                      "{% endfor %}",
                                      "{% else %}",
                                      "trailing tags"))
            .unwrap();

        // note that we need an options block that has been initilaised with
        // the supported block list; otherwise the split_tag function won't know
        // which things start a nested block.
        let options = options();
        let (_, trailing) = split_block(&tokens[..], &["else"], &options);
        match trailing {
            Some(split) => {
                assert_eq!(split.delimiter, "else");
                assert_eq!(split.args, &[Token::Identifier("else".to_owned())]);
                assert_eq!(split.trailing,
                           &[Element::Tag(vec![Token::Identifier("else".to_owned())],
                                          "{% else %}".to_owned()),
                             Element::Raw("trailing tags".to_owned())]);
            }
            None => panic!("split failed"),
        }
    }
}
