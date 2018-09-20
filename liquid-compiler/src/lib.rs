#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate liquid_error;
extern crate liquid_value;
extern crate liquid_interpreter;

// Minimize retrofits
mod interpreter {
    pub(crate) use liquid_interpreter::*;
}
mod value {
    pub(crate) use liquid_value::*;
}

mod block;
mod include;
mod lexer;
mod options;
mod parser;
mod tag;
mod token;

pub use liquid_error::{Error, Result, ResultLiquidChainExt, ResultLiquidExt};

pub use self::block::{BoxedBlockParser, FnParseBlock, ParseBlock, ParseBlockClone};
pub use self::include::{FilesystemInclude, Include, IncludeClone, NullInclude};
pub use self::lexer::{tokenize, Element};
pub use self::options::LiquidOptions;
pub use self::parser::{
    consume_value_token, expect, parse, parse_indexes, parse_output, split_block,
    unexpected_token_error, value_token, BlockSplit,
};
pub use self::tag::{BoxedTagParser, FnParseTag, ParseTag, ParseTagClone};
pub use self::token::{ComparisonOperator, Token};
