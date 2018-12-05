use liquid_interpreter::PluginRegistry;

use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::Include;
use super::NullInclude;

#[derive(Clone)]
pub struct LiquidOptions {
    pub blocks: PluginRegistry<BoxedBlockParser>,
    pub tags: PluginRegistry<BoxedTagParser>,
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
