use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::BoxedValueFilter;
use super::Include;
use super::NullInclude;
use super::PluginRegistry;

#[derive(Clone)]
pub struct LiquidOptions {
    pub blocks: PluginRegistry<BoxedBlockParser>,
    pub tags: PluginRegistry<BoxedTagParser>,
    pub filters: PluginRegistry<BoxedValueFilter>,
    pub include_source: Box<Include>,
}

impl Default for LiquidOptions {
    fn default() -> LiquidOptions {
        LiquidOptions {
            blocks: Default::default(),
            tags: Default::default(),
            filters: Default::default(),
            include_source: Box::new(NullInclude::new()),
        }
    }
}
