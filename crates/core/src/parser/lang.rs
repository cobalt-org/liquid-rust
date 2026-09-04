use super::ParseBlock;
use super::ParseFilter;
use super::ParseTag;
use super::PluginRegistry;

#[derive(Clone)]
pub enum ParseMode {
    Strict,
    Lax,
}

impl Default for ParseMode {
    fn default() -> Self {
        Self::Strict
    }
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct Language {
    pub blocks: PluginRegistry<Box<dyn ParseBlock>>,
    pub tags: PluginRegistry<Box<dyn ParseTag>>,
    pub filters: PluginRegistry<Box<dyn ParseFilter>>,
    pub mode: ParseMode,
}

impl Language {
    pub fn empty() -> Self {
        Default::default()
    }
}
