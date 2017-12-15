use std::collections::HashMap;

use super::ParseBlock;
use super::ParseTag;
use super::Include;
use super::NullInclude;

#[derive(Clone)]
pub struct LiquidOptions {
    pub blocks: HashMap<String, Box<ParseBlock>>,
    pub tags: HashMap<String, Box<ParseTag>>,
    pub include_source: Box<Include>,
}

impl Default for LiquidOptions {
    fn default() -> LiquidOptions {
        LiquidOptions {
            blocks: Default::default(),
            tags: Default::default(),
            include_source: Box::new(NullInclude::new()),
        }
    }
}
