use super::ParseBlock;
use super::ParseFilter;
use super::ParseTag;
use super::PluginRegistry;

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct Language {
    pub blocks: PluginRegistry<Box<dyn ParseBlock>>,
    pub tags: PluginRegistry<Box<dyn ParseTag>>,
    pub filters: PluginRegistry<Box<dyn ParseFilter>>,
}

impl Language {
    pub fn empty() -> Self {
        Default::default()
    }
}
