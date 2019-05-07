use super::ParseBlock;
use super::ParseFilter;
use super::ParseTag;
use super::PluginRegistry;

#[derive(Clone)]
pub struct Language {
    pub blocks: PluginRegistry<Box<ParseBlock>>,
    pub tags: PluginRegistry<Box<ParseTag>>,
    pub filters: PluginRegistry<Box<ParseFilter>>,
    non_exhaustive: (),
}

impl Language {
    pub fn empty() -> Self {
        Default::default()
    }
}

impl Default for Language {
    fn default() -> Language {
        Language {
            blocks: Default::default(),
            tags: Default::default(),
            filters: Default::default(),
            non_exhaustive: Default::default(),
        }
    }
}
