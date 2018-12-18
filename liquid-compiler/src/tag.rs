use liquid_error::Result;
use liquid_interpreter::Renderable;

use super::Language;
use super::TagTokenIter;

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [`Language`](struct.Language.html).
pub trait ParseTag: Send + Sync + ParseTagClone {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        options: &Language,
    ) -> Result<Box<Renderable>>;
}

pub trait ParseTagClone {
    fn clone_box(&self) -> Box<ParseTag>;
}

impl<T> ParseTagClone for T
where
    T: 'static + ParseTag + Clone,
{
    fn clone_box(&self) -> Box<ParseTag> {
        Box::new(self.clone())
    }
}

impl Clone for Box<ParseTag> {
    fn clone(&self) -> Box<ParseTag> {
        self.clone_box()
    }
}
pub type FnParseTag = fn(&str, TagTokenIter, &Language) -> Result<Box<Renderable>>;

#[derive(Clone)]
struct FnTagParser {
    parser: FnParseTag,
}

impl FnTagParser {
    fn new(parser: FnParseTag) -> Self {
        Self { parser }
    }
}

impl ParseTag for FnTagParser {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        options: &Language,
    ) -> Result<Box<Renderable>> {
        (self.parser)(tag_name, arguments, options)
    }
}

#[derive(Clone)]
enum TagParserEnum {
    Fun(FnTagParser),
    Heap(Box<ParseTag>),
}

#[derive(Clone)]
pub struct BoxedTagParser {
    parser: TagParserEnum,
}

impl ParseTag for BoxedTagParser {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        options: &Language,
    ) -> Result<Box<Renderable>> {
        match self.parser {
            TagParserEnum::Fun(ref f) => f.parse(tag_name, arguments, options),
            TagParserEnum::Heap(ref f) => f.parse(tag_name, arguments, options),
        }
    }
}

impl From<FnParseTag> for BoxedTagParser {
    fn from(parser: FnParseTag) -> BoxedTagParser {
        let parser = TagParserEnum::Fun(FnTagParser::new(parser));
        Self { parser }
    }
}

impl From<Box<ParseTag>> for BoxedTagParser {
    fn from(parser: Box<ParseTag>) -> BoxedTagParser {
        let parser = TagParserEnum::Heap(parser);
        Self { parser }
    }
}
