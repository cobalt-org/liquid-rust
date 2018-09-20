use std::collections::HashMap;

use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::Include;
use super::NullInclude;

#[derive(Clone)]
pub struct LiquidOptions {
    pub blocks: HashMap<&'static str, BoxedBlockParser>,
    pub tags: HashMap<&'static str, BoxedTagParser>,
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
