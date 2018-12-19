use super::BoxedBlockParser;
use super::BoxedTagParser;
use super::BoxedValueFilter;
use super::Include;
use super::PluginRegistry;

#[derive(Clone)]
pub struct Language {
    pub blocks: PluginRegistry<BoxedBlockParser>,
    pub tags: PluginRegistry<BoxedTagParser>,
    pub filters: PluginRegistry<BoxedValueFilter>,
    pub include_source: Option<Box<Include>>,
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
            include_source: None,
            non_exhaustive: Default::default(),
        }
    }
}
