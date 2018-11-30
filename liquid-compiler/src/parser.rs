//! Parser
//!
//! This module contains functions than can be used for writing plugins
//! but should be ignored for simple usage.

use std;

use liquid_interpreter::Expression;
use liquid_interpreter::Renderable;
use liquid_interpreter::Text;
use liquid_interpreter::Variable;
use liquid_interpreter::{FilterCall, FilterChain};
use liquid_value::Scalar;
use liquid_value::Value;

use super::error::{Error, Result};
use super::LiquidOptions;
use super::ParseBlock;
use super::ParseTag;

use pest::Parser;

mod pest {
    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct LiquidParser;
}

use self::pest::*;

type Pair<'a> = ::pest::iterators::Pair<'a, Rule>;
type Pairs<'a> = ::pest::iterators::Pairs<'a, Rule>;

/// Converts a `pest::Error` into a `liquid::Error`.
fn convert_pest_error(err: ::pest::error::Error<Rule>) -> Error {
    let err = err.renamed_rules(|&rule| match rule {
        Rule::LesserThan => "\"<\"".to_string(),
        Rule::GreaterThan => "\">\"".to_string(),
        Rule::LesserThanEquals => "\"<=\"".to_string(),
        Rule::GreaterThanEquals => "\">=\"".to_string(),
        Rule::Equals => "\"==\"".to_string(),
        Rule::NotEquals => "\"!=\"".to_string(),
        Rule::LesserThanGreaterThan => "\"<>\"".to_string(),
        Rule::Assign => "\"=\"".to_string(),
        Rule::Comma => "\",\"".to_string(),
        Rule::Colon => "\":\"".to_string(),
        other => format!("{:?}", other),
    });
    Error::with_msg(err.to_string())
}

/// Generates a `liquid::Error` with the given message pointing to
/// the pest
fn error_from_pair(pair: Pair, msg: String) -> Error {
    let pest_error = ::pest::error::Error::new_from_span(
        ::pest::error::ErrorVariant::CustomError { message: msg },
        pair.as_span(),
    );
    convert_pest_error(pest_error)
}

/// Parses the provided &str into a number of Renderable items.
pub fn parse(text: &str, options: &LiquidOptions) -> Result<Vec<Box<Renderable>>> {
    let mut liquid = LiquidParser::parse(Rule::LiquidFile, text)
        .map_err(convert_pest_error)?
        .next()
        .expect("Unwrapping LiquidFile to access the elements.")
        .into_inner();

    let mut renderables = Vec::new();

    while let Some(element) = liquid.next() {
        if element.as_rule() == Rule::EOI {
            break;
        }

        renderables.push(BlockElement::parse_pair(
            element.into(),
            &mut liquid,
            options,
        )?);
    }
    Ok(renderables)
}

/// Parses a `Scalar` from a `Pair` with a literal value.
/// This `Pair` must be `Rule::Literal`.
fn parse_literal(literal: Pair) -> Scalar {
    if literal.as_rule() != Rule::Literal {
        panic!("Expected literal.");
    }

    let literal = literal
        .into_inner()
        .next()
        .expect("Get into the rule inside literal.");

    match literal.as_rule() {
        Rule::StringLiteral => {
            let literal = literal.as_str();
            let trim_quotes = &literal[1..literal.len() - 1];

            Scalar::new(trim_quotes.to_owned())
        }
        Rule::IntegerLiteral => Scalar::new(
            literal
                .as_str()
                .parse::<i32>()
                .expect("Matches are parseable as integers."),
        ),
        Rule::FloatLiteral => Scalar::new(
            literal
                .as_str()
                .parse::<f64>()
                .expect("Matches are parseable as floats."),
        ),
        Rule::BooleanLiteral => Scalar::new(
            literal
                .as_str()
                .parse::<bool>()
                .expect("Matches are parseable as bools."),
        ),
        _ => unreachable!(),
    }
}

/// Parses a `Variable` from a `Pair` with a variable.
/// This `Pair` must be `Rule::Variable`.
fn parse_variable(variable: Pair) -> Variable {
    if variable.as_rule() != Rule::Variable {
        panic!("Expected variable.");
    }

    let mut indexes = variable.into_inner();

    let first_identifier = indexes
        .next()
        .expect("A variable starts with an identifier.")
        .as_str()
        .to_owned();
    let mut variable = Variable::with_literal(first_identifier);

    let indexes = indexes.map(|index| match index.as_rule() {
        Rule::Identifier => Expression::with_literal(index.as_str().to_owned()),
        Rule::Value => parse_value(index),
        _ => unreachable!(),
    });

    variable.extend(indexes);
    variable
}

/// Parses an `Expression` from a `Pair` with a value.
///
/// Do not confuse this value with `liquid-value`'s `Value`.
/// In this context, value refers to either a literal value or a variable.
///
/// This `Pair` must be `Rule::Value`.
fn parse_value(value: Pair) -> Expression {
    if value.as_rule() != Rule::Value {
        panic!("Expected value.");
    }

    let value = value.into_inner().next().expect("Get inside the value.");

    match value.as_rule() {
        Rule::Literal => Expression::with_literal(parse_literal(value)),
        Rule::Variable => Expression::Variable(parse_variable(value)),
        _ => unreachable!(),
    }
}

/// Parses a `FilterCall` from a `Pair` with a filter.
/// This `Pair` must be `Rule::Filter`.
fn parse_filter(filter: Pair) -> FilterCall {
    if filter.as_rule() != Rule::Filter {
        panic!("Expected a filter.");
    }

    let mut filter = filter.into_inner();
    let name = filter.next().expect("A filter always has a name.").as_str();
    let args = filter.map(parse_value).collect();

    FilterCall::new(name, args)
}

/// Parses a `FilterChain` from a `Pair` with a filter chain.
/// This `Pair` must be `Rule::FilterChain`.
fn parse_filter_chain(chain: Pair) -> FilterChain {
    if chain.as_rule() != Rule::FilterChain {
        panic!("Expected an expression with filters.");
    }

    let mut chain = chain.into_inner();
    let entry = parse_value(
        chain
            .next()
            .expect("A filterchain always has starts by a value."),
    );
    let filters = chain.map(parse_filter).collect();

    FilterChain::new(entry, filters)
}

/// An interface to access elements inside a block.
pub struct TagBlock<'a: 'b, 'b> {
    name: &'b str,
    iter: &'b mut Iterator<Item = Pair<'a>>,
    closed: bool,
}

impl<'a, 'b> TagBlock<'a, 'b> {
    fn new(name: &'b str, next_elements: &'b mut Iterator<Item = Pair<'a>>) -> Self {
        TagBlock {
            name,
            iter: next_elements,
            closed: false,
        }
    }

    /// Returns the next element of the block, if any, similarly to an iterator.
    ///
    /// However, if the input text reaches its end and the block is not closed,
    /// an error is returned instead.
    pub fn next(&mut self) -> Result<Option<BlockElement<'a>>> {
        if self.closed {
            return Ok(None);
        }

        let element = self.iter.next().expect("File shouldn't end before EOI.");

        if element.as_rule() == Rule::EOI {
            return Err(error_from_pair(
                element,
                format!("Unclosed block. {{% end{} %}} tag expected.", self.name),
            ));
        }

        // Tags are treated separately so as to check for a possible `{% endtag %}`
        if element.as_rule() == Rule::Tag {
            let as_str = element.as_str();
            let mut tag = element
                .into_inner()
                .next()
                .expect("Unwrapping TagInner")
                .into_inner();
            let name = tag.next().expect("Tags start by their identifier.");
            let name_str = name.as_str();

            // The name of the closing tag is "end" followed by the tag's name.
            if name_str.len() > 3 && &name_str[0..3] == "end" && &name_str[3..] == self.name {
                // Then this is a block ending tag and will close the block.

                // no more arguments should be supplied, trying to supply them is an error
                if let Some(token) = tag.next() {
                    return Err(TagToken::from(token).raise_error());
                }

                self.closed = true;
                return Ok(None);
            } else {
                // Then this is a regular tag
                let tokens = TagTokenIter::new(&name, tag);
                return Ok(Some(BlockElement::Tag(Tag {
                    name,
                    tokens,
                    as_str,
                })));
            }
        }
        Ok(Some(element.into()))
    }

    /// A convenient method that parses every element remaining in the block.
    pub fn parse_all(&mut self, options: &LiquidOptions) -> Result<Vec<Box<Renderable>>> {
        let mut renderables = Vec::new();
        while let Some(r) = self.parse_next(options)? {
            renderables.push(r);
        }
        Ok(renderables)
    }

    /// Parses the next element in the block just as if it weren't inside any block.
    ///
    /// Returns none if no element is left and raises the same errors as `next()`.
    pub fn parse_next(&mut self, options: &LiquidOptions) -> Result<Option<Box<Renderable>>> {
        match self.next()? {
            None => Ok(None),
            Some(element) => Ok(Some(element.parse(self, options)?)),
        }
    }

    /// Checks whether the block was fully parsed its elements.
    ///
    /// This must be added at the end of every block right before returning, so as
    /// to ensure that it doesn't leave any unparsed element by accident.
    pub fn assert_empty(self) {
        assert!(
            self.closed,
            "Block {{% {} %}} doesn't exhaust its iterator of elements.",
            self.name
        )
    }
}

/// An element that is raw text.
pub struct Raw<'a> {
    text: &'a str,
}
impl<'a> From<Pair<'a>> for Raw<'a> {
    fn from(element: Pair<'a>) -> Self {
        if element.as_rule() != Rule::Raw {
            panic!("Only rule Raw can be converted to Raw.");
        }
        Raw {
            text: element.as_str(),
        }
    }
}
impl<'a> Into<&'a str> for Raw<'a> {
    fn into(self) -> &'a str {
        self.as_str()
    }
}
impl<'a> Raw<'a> {
    /// Turns the text into a Renderable.
    pub fn to_renderable(self) -> Box<Renderable> {
        Box::new(Text::new(self.as_str()))
    }

    /// Returns the text as a str.
    pub fn as_str(&self) -> &'a str {
        self.text
    }
}

/// An element that is a tag.
pub struct Tag<'a> {
    name: Pair<'a>,
    tokens: TagTokenIter<'a>,
    as_str: &'a str,
}
impl<'a> From<Pair<'a>> for Tag<'a> {
    fn from(element: Pair<'a>) -> Self {
        if element.as_rule() != Rule::Tag {
            panic!("Only rule Tag can be converted to Tag.");
        }
        let as_str = element.as_str();
        let mut tag = element
            .into_inner()
            .next()
            .expect("Unwrapping TagInner.")
            .into_inner();
        let name = tag.next().expect("A tag starts with an identifier.");
        let tokens = TagTokenIter::new(&name, tag);

        Tag {
            name,
            tokens,
            as_str,
        }
    }
}

impl<'a> Tag<'a> {
    /// Creates a new tag from a string such as "{% tagname tagtoken1 tagtoken2 ... %}".
    ///
    /// This is used as a debug tool. It allows to easily build tags in unit tests.
    pub fn new(text: &'a str) -> Result<Self> {
        let tag = LiquidParser::parse(Rule::Tag, text)
            .map_err(convert_pest_error)?
            .next()
            .ok_or_else(|| Error::with_msg("Tried to create a Tag from an invalid string."))?;

        Ok(tag.into())
    }

    /// Returns the name of this tag.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the tokens of this tag.
    pub fn tokens(&mut self) -> &mut TagTokenIter<'a> {
        &mut self.tokens
    }

    /// Consumes this structure to obtain ownership over its tokens.
    pub fn into_tokens(self) -> TagTokenIter<'a> {
        self.tokens
    }

    /// Returns the tag as a str.
    pub fn as_str(&self) -> &str {
        self.as_str
    }

    /// Parses the tag just as if it weren't inside any block.
    pub fn parse(
        self,
        tag_block: &mut TagBlock,
        options: &LiquidOptions,
    ) -> Result<Box<Renderable>> {
        self.parse_pair(&mut tag_block.iter, options)
    }

    /// The same as `parse`, but directly takes an iterator over `Pair`s instead of a TagBlock.
    fn parse_pair(
        self,
        next_elements: &mut Iterator<Item = Pair>,
        options: &LiquidOptions,
    ) -> Result<Box<Renderable>> {
        let (name, tokens) = (self.name, self.tokens);
        let position = name.as_span();
        let name = name.as_str();

        if options.tags.contains_key(name) {
            options.tags[name].parse(name, tokens, options)
        } else if options.blocks.contains_key(name) {
            let block = TagBlock::new(name, next_elements);
            let renderables = options.blocks[name].parse(name, tokens, block, options)?;
            Ok(renderables)
        } else {
            let pest_error = ::pest::error::Error::new_from_span(
                ::pest::error::ErrorVariant::CustomError {
                    message: "Unknown tag.".to_string(),
                },
                position,
            );
            Err(convert_pest_error(pest_error))
        }
    }
}

/// An element that is an expression.
pub struct Exp<'a> {
    element: Pair<'a>,
}
impl<'a> From<Pair<'a>> for Exp<'a> {
    fn from(element: Pair<'a>) -> Self {
        if element.as_rule() != Rule::Expression {
            panic!("Only rule Expression can be converted to Expression.");
        }
        Exp { element }
    }
}
impl<'a> Exp<'a> {
    /// Parses the expression just as if it weren't inside any block.
    pub fn parse(self) -> Result<Box<Renderable>> {
        let filter_chain = self
            .element
            .into_inner()
            .next()
            .expect("Unwrapping ExpressionInner")
            .into_inner()
            .next()
            .expect("An expression consists of one filterchain.");

        Ok(Box::new(parse_filter_chain(filter_chain)))
    }

    /// Returns the expression as a str.
    pub fn as_str(&self) -> &str {
        self.element.as_str()
    }
}

/// An element that can be raw text, a tag, or an expression.
///
/// This is the result of calling `next()` on a `TagBlock`.
pub enum BlockElement<'a> {
    Raw(Raw<'a>),
    Tag(Tag<'a>),
    Expression(Exp<'a>),
}
impl<'a> From<Pair<'a>> for BlockElement<'a> {
    fn from(element: Pair<'a>) -> Self {
        match element.as_rule() {
            Rule::Raw => BlockElement::Raw(element.into()),
            Rule::Tag => BlockElement::Tag(element.into()),
            Rule::Expression => BlockElement::Expression(element.into()),
            _ => panic!(
                "Only rules Raw | Tag | Expression can be converted to BlockElement. Found {:?}",
                element.as_rule()
            ),
        }
    }
}

impl<'a> BlockElement<'a> {
    /// Parses the element in the block just as if it weren't inside any block.
    pub fn parse(
        self,
        block: &mut TagBlock<'a, '_>,
        options: &LiquidOptions,
    ) -> Result<Box<Renderable>> {
        match self {
            BlockElement::Raw(raw) => Ok(raw.to_renderable()),
            BlockElement::Tag(tag) => tag.parse(block, options),
            BlockElement::Expression(exp) => exp.parse(),
        }
    }

    /// The same as `parse`, but directly takes an iterator over `Pair`s instead of a TagBlock.
    fn parse_pair(
        self,
        next_elements: &mut Iterator<Item = Pair>,
        options: &LiquidOptions,
    ) -> Result<Box<Renderable>> {
        match self {
            BlockElement::Raw(raw) => Ok(raw.to_renderable()),
            BlockElement::Tag(tag) => tag.parse_pair(next_elements, options),
            BlockElement::Expression(exp) => exp.parse(),
        }
    }

    /// Returns the element as a str.
    pub fn as_str(&self) -> &str {
        match self {
            BlockElement::Raw(raw) => raw.as_str(),
            BlockElement::Tag(tag) => tag.as_str(),
            BlockElement::Expression(exp) => exp.as_str(),
        }
    }
}

/// An iterator over `TagToken`s that is aware of their position in the file.
///
/// The awareness of the position allows more precise error messages.
pub struct TagTokenIter<'a> {
    iter: Box<Iterator<Item = TagToken<'a>> + 'a>,
    position: ::pest::Position<'a>,
}
impl<'a> Iterator for TagTokenIter<'a> {
    type Item = TagToken<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|next| {
            self.position = next.token.as_span().end_pos();
            next
        })
    }
}
impl<'a> TagTokenIter<'a> {
    fn new(name: &Pair<'a>, tokens: Pairs<'a>) -> Self {
        TagTokenIter {
            iter: Box::new(tokens.map(TagToken::from)),
            position: name.as_span().end_pos(),
        }
    }

    /// Creates an error with the given message pointing at the current
    /// position of the iterator.
    pub fn raise_error(&mut self, error_msg: &str) -> Error {
        let pest_error = ::pest::error::Error::new_from_pos(
            ::pest::error::ErrorVariant::CustomError {
                message: error_msg.to_string(),
            },
            self.position.clone(),
        );
        convert_pest_error(pest_error)
    }

    /// Returns the next tag token or raises an error if there is none.
    pub fn expect_next(&mut self, error_msg: &str) -> Result<TagToken<'a>> {
        self.next().ok_or_else(|| self.raise_error(error_msg))
    }

    /// Returns `Ok` if the iterator is empty, an error otherwise
    pub fn expect_nothing(&mut self) -> Result<()> {
        if let Some(token) = self.next() {
            Err(token.raise_error())
        } else {
            Ok(())
        }
    }
}

/// The result of calling `TagToken`'s `try`.
///
/// If the token is successfuly matched, the match is returned;
/// otherwise, the TagToken is returned back.
pub enum TryMatchToken<'a, T> {
    Matches(T),
    Fails(TagToken<'a>),
}

impl<'a, T> TryMatchToken<'a, T> {
    pub fn into_result(self) -> Result<T> {
        match self {
            TryMatchToken::Matches(t) => Ok(t),
            TryMatchToken::Fails(t) => Err(t.raise_error()),
        }
    }

    pub fn into_result_custom_msg(self, msg: &str) -> Result<T> {
        match self {
            TryMatchToken::Matches(t) => Ok(t),
            TryMatchToken::Fails(t) => Err(t.raise_custom_error(msg)),
        }
    }
}

/// An interface to access tokens inside a tag.
pub struct TagToken<'a> {
    token: Pair<'a>,
    expected: Vec<Rule>,
}

impl<'a> From<Pair<'a>> for TagToken<'a> {
    fn from(token: Pair<'a>) -> Self {
        TagToken {
            token,
            expected: Vec::new(),
        }
    }
}

impl<'a> TagToken<'a> {
    /// Raises an error from this TagToken.
    ///
    /// The error message will be based on the expected tokens,
    /// which this structure tracks when using the methods starting
    /// with 'expect'.
    ///
    /// For example, if one calls `expect_value` and that function fails
    /// to give an `Ok` value, calling this would show `Expected Value`
    /// on the error message.
    pub fn raise_error(self) -> Error {
        let pest_error = ::pest::error::Error::new_from_span(
            ::pest::error::ErrorVariant::ParsingError {
                positives: self.expected,
                negatives: vec![self.token.as_rule()],
            },
            self.token.as_span(),
        );
        convert_pest_error(pest_error)
    }

    /// Raises an error from this TagToken.
    ///
    /// The error will have the given error message.
    pub fn raise_custom_error(self, msg: &str) -> Error {
        let pest_error = ::pest::error::Error::new_from_span(
            ::pest::error::ErrorVariant::CustomError {
                message: msg.to_string(),
            },
            self.token.as_span(),
        );
        convert_pest_error(pest_error)
    }

    fn unwrap_filter_chain(&mut self) -> std::result::Result<Pair<'a>, ()> {
        let token = self.token.clone();

        if token.as_rule() != Rule::FilterChain {
            return Err(());
        }

        Ok(token)
    }

    fn unwrap_value(&mut self) -> std::result::Result<Pair<'a>, ()> {
        let filterchain = self.unwrap_filter_chain()?;

        let mut chain = filterchain.into_inner();
        let value = chain.next().expect("Unwrapping value out of Filterchain.");
        if chain.next().is_some() {
            // There are filters: it can't be a value
            return Err(());
        }

        Ok(value)
    }

    fn unwrap_variable(&mut self) -> std::result::Result<Pair<'a>, ()> {
        let value = self.unwrap_value()?;

        let variable = value
            .into_inner()
            .next()
            .expect("A value is made of one token.");

        if variable.as_rule() != Rule::Variable {
            return Err(());
        }

        Ok(variable)
    }

    fn unwrap_identifier(&mut self) -> std::result::Result<Pair<'a>, ()> {
        let variable = self.unwrap_variable()?;

        let mut indexes = variable.into_inner();
        let identifier = indexes
            .next()
            .expect("Unwrapping identifier out of variable.");
        if indexes.next().is_some() {
            // There are indexes: it can't be a value
            return Err(());
        }

        Ok(identifier)
    }

    fn unwrap_literal(&mut self) -> std::result::Result<Pair<'a>, ()> {
        let value = self.unwrap_value()?;

        let literal = value
            .into_inner()
            .next()
            .expect("A value is made of one token.");

        if literal.as_rule() != Rule::Literal {
            return Err(());
        }

        Ok(literal)
    }

    /// Tries to obtain a `FilterChain` from this token.
    pub fn expect_filter_chain(mut self) -> TryMatchToken<'a, FilterChain> {
        match self.unwrap_filter_chain() {
            Ok(t) => TryMatchToken::Matches(parse_filter_chain(t)),
            Err(_) => {
                self.expected.push(Rule::FilterChain);
                TryMatchToken::Fails(self)
            }
        }
    }

    /// Tries to obtain a value from this token.
    ///
    /// Do not confuse this value with `liquid-value`'s `Value`.
    /// In this context, value refers to either a literal value or a variable.
    pub fn expect_value(mut self) -> TryMatchToken<'a, Expression> {
        match self.unwrap_value() {
            Ok(t) => TryMatchToken::Matches(parse_value(t)),
            Err(_) => {
                self.expected.push(Rule::Value);
                TryMatchToken::Fails(self)
            }
        }
    }

    /// Tries to obtain a `Variable` from this token.
    pub fn expect_variable(mut self) -> TryMatchToken<'a, Variable> {
        match self.unwrap_variable() {
            Ok(t) => TryMatchToken::Matches(parse_variable(t)),
            Err(_) => {
                self.expected.push(Rule::Variable);
                TryMatchToken::Fails(self)
            }
        }
    }

    /// Tries to obtain an identifier from this token.
    ///
    /// The identifier is returned as a str.
    pub fn expect_identifier(mut self) -> TryMatchToken<'a, &'a str> {
        match self.unwrap_identifier() {
            Ok(t) => TryMatchToken::Matches(t.as_str()),
            Err(_) => {
                self.expected.push(Rule::Identifier);
                TryMatchToken::Fails(self)
            }
        }
    }

    /// Tries to obtain a literal value from this token.
    ///
    /// The value is returned as a `Value`.
    pub fn expect_literal(mut self) -> TryMatchToken<'a, Value> {
        match self.unwrap_literal() {
            Ok(t) => TryMatchToken::Matches(Value::scalar(parse_literal(t))),
            Err(_) => {
                self.expected.push(Rule::Literal);
                TryMatchToken::Fails(self)
            }
        }
    }
    /// Tries to obtain a range from this token.
    ///
    /// The range is returned as a pair `(Expression, Expression)`.
    pub fn expect_range(mut self) -> TryMatchToken<'a, (Expression, Expression)> {
        let token = self.token.clone();

        if token.as_rule() != Rule::Range {
            self.expected.push(Rule::Range);
            return TryMatchToken::Fails(self);
        }

        let mut range = token.into_inner();
        TryMatchToken::Matches((
            parse_value(range.next().expect("start")),
            parse_value(range.next().expect("end")),
        ))
    }

    /// Returns `Ok` if and only if the tokens' str is equal to the given str.
    pub fn expect_str(self, expected: &str) -> TryMatchToken<'a, ()> {
        if self.as_str() == expected {
            TryMatchToken::Matches(())
        } else {
            // TODO change `self`'s state to be aware that `expected` was expected.
            TryMatchToken::Fails(self)
        }
    }

    /// Returns token as a str.
    pub fn as_str(&self) -> &str {
        self.token.as_str().trim()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use liquid_interpreter::{Context, Template};

    #[test]
    fn test_parse_literal() {
        let integer = LiquidParser::parse(Rule::Literal, "42")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(integer), Scalar::new(42));

        let negative_int = LiquidParser::parse(Rule::Literal, "-42")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(negative_int), Scalar::new(-42));

        let float = LiquidParser::parse(Rule::Literal, "4321.032")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(float), Scalar::new(4321.032));

        let negative_float = LiquidParser::parse(Rule::Literal, "-4321.032")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(negative_float), Scalar::new(-4321.032));

        let boolean = LiquidParser::parse(Rule::Literal, "true")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(boolean), Scalar::new(true));

        let string_double_quotes = LiquidParser::parse(Rule::Literal, "\"Hello world!\"")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(
            parse_literal(string_double_quotes),
            Scalar::new("Hello world!")
        );

        let string_single_quotes = LiquidParser::parse(Rule::Literal, "'Liquid'")
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(parse_literal(string_single_quotes), Scalar::new("Liquid"));
    }

    #[test]
    fn test_parse_filter_chain() {
        let filter = LiquidParser::parse(Rule::FilterChain, "abc | def:'1',2,'3' | blabla")
            .unwrap()
            .next()
            .unwrap();

        assert_eq!(
            parse_filter_chain(filter),
            FilterChain::new(
                Expression::Variable(Variable::with_literal("abc")),
                vec![
                    FilterCall::new(
                        "def",
                        vec![
                            Expression::Literal(Value::scalar("1")),
                            Expression::Literal(Value::scalar(2.0)),
                            Expression::Literal(Value::scalar("3")),
                        ],
                    ),
                    FilterCall::new("blabla", vec![]),
                ]
            )
        );
    }

    #[test]
    fn test_parse_variable() {
        let variable = LiquidParser::parse(Rule::Variable, "foo[0].bar.baz[foo.bar]")
            .unwrap()
            .next()
            .unwrap();

        let indexes = vec![
            Expression::Literal(Value::scalar(0)),
            Expression::Literal(Value::scalar("bar")),
            Expression::Literal(Value::scalar("baz")),
            Expression::Variable(Variable::with_literal("foo").push_literal("bar")),
        ];

        let mut expected = Variable::with_literal("foo");
        expected.extend(indexes);

        assert_eq!(parse_variable(variable), expected);
    }

    #[test]
    fn test_whitespace_control() {
        let options = LiquidOptions::default();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("exp", Value::Scalar(Scalar::new(5)));

        let text = "    \n    {{ exp }}    \n    ";
        let template = parse(text, &options).map(Template::new).unwrap();
        let output = template.render(&mut context).unwrap();

        assert_eq!(output, "    \n    5    \n    ");

        let text = "    \n    {{- exp }}    \n    ";
        let template = parse(text, &options).map(Template::new).unwrap();
        let output = template.render(&mut context).unwrap();

        assert_eq!(output, "5    \n    ");

        let text = "    \n    {{ exp -}}    \n    ";
        let template = parse(text, &options).map(Template::new).unwrap();
        let output = template.render(&mut context).unwrap();

        assert_eq!(output, "    \n    5");

        let text = "    \n    {{- exp -}}    \n    ";
        let template = parse(text, &options).map(Template::new).unwrap();
        let output = template.render(&mut context).unwrap();

        assert_eq!(output, "5");
    }
}
