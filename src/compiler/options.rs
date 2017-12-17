use std::collections::HashMap;

use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::Include;
use super::NullInclude;

#[derive(Clone)]
pub struct LiquidOptions {
    pub blocks: HashMap<String, BoxedBlockParser>,
    pub tags: HashMap<String, BoxedTagParser>,
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
