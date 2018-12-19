use liquid_error::Result;
use liquid_interpreter::Renderable;

use super::Language;
use super::TagBlock;
use super::TagTokenIter;

/// A trait for creating custom custom block-size tags (`{% if something %}{% endif %}`).
/// This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a block and returns
/// a new `Renderable` based on its parameters. The received parameters specify the name
/// of the block, the argument [Tokens](lexer/enum.Token.html) passed to
/// the block, a Vec of all [Elements](lexer/enum.Element.html) inside the block and
/// the global [`Language`](struct.Language.html).
pub trait ParseBlock: Send + Sync + ParseBlockClone {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        block: TagBlock,
        options: &Language,
    ) -> Result<Box<Renderable>>;
}

pub trait ParseBlockClone {
    fn clone_box(&self) -> Box<ParseBlock>;
}

impl<T> ParseBlockClone for T
where
    T: 'static + ParseBlock + Clone,
{
    fn clone_box(&self) -> Box<ParseBlock> {
        Box::new(self.clone())
    }
}

impl Clone for Box<ParseBlock> {
    fn clone(&self) -> Box<ParseBlock> {
        self.clone_box()
    }
}

pub type FnParseBlock = fn(&str, TagTokenIter, TagBlock, &Language) -> Result<Box<Renderable>>;

#[derive(Clone)]
struct FnBlockParser {
    parser: FnParseBlock,
}

impl FnBlockParser {
    fn new(parser: FnParseBlock) -> Self {
        Self { parser }
    }
}

impl ParseBlock for FnBlockParser {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        tokens: TagBlock,
        options: &Language,
    ) -> Result<Box<Renderable>> {
        (self.parser)(tag_name, arguments, tokens, options)
    }
}

#[derive(Clone)]
enum BlockParserEnum {
    Fun(FnBlockParser),
    Heap(Box<ParseBlock>),
}

#[derive(Clone)]
pub struct BoxedBlockParser {
    parser: BlockParserEnum,
}

impl ParseBlock for BoxedBlockParser {
    fn parse(
        &self,
        tag_name: &str,
        arguments: TagTokenIter,
        tokens: TagBlock,
        options: &Language,
    ) -> Result<Box<Renderable>> {
        match self.parser {
            BlockParserEnum::Fun(ref f) => f.parse(tag_name, arguments, tokens, options),
            BlockParserEnum::Heap(ref f) => f.parse(tag_name, arguments, tokens, options),
        }
    }
}

impl From<FnParseBlock> for BoxedBlockParser {
    fn from(parser: FnParseBlock) -> BoxedBlockParser {
        let parser = BlockParserEnum::Fun(FnBlockParser::new(parser));
        Self { parser }
    }
}

impl From<Box<ParseBlock>> for BoxedBlockParser {
    fn from(parser: Box<ParseBlock>) -> BoxedBlockParser {
        let parser = BlockParserEnum::Heap(parser);
        Self { parser }
    }
}
