use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::BoxedValueFilter;
use super::PluginRegistry;

#[derive(Clone)]
pub struct Language {
    pub blocks: PluginRegistry<BoxedBlockParser>,
    pub tags: PluginRegistry<BoxedTagParser>,
    pub filters: PluginRegistry<BoxedValueFilter>,
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
