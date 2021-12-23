use std::collections::HashMap;
use std::fmt;
use std::sync;

use crate::error::Result;
use crate::parser;
use crate::parser::Language;
use crate::runtime;
use crate::runtime::PartialStore;
use crate::runtime::Renderable;

use super::PartialCompiler;
use super::PartialSource;

/// An lazily-caching compiler for `PartialSource`.
///
/// This would be useful in cases where:
/// - Most partial-templates aren't used
/// - Of the used partial-templates, they are generally used many times.
///
/// Note: partial-compilation error reporting is deferred to render-time so content can still be
/// generated even when the content is in an intermediate-state.
#[derive(Debug)]
pub struct LazyCompiler<S: PartialSource> {
    source: S,
}

impl<S> LazyCompiler<S>
where
    S: PartialSource,
{
    /// Create an on-demand compiler for `PartialSource`.
    pub fn new(source: S) -> Self {
        LazyCompiler { source }
    }
}

impl<S> LazyCompiler<S>
where
    S: PartialSource + Default,
{
    /// Create an empty compiler for `PartialSource`.
    pub fn empty() -> Self {
        Default::default()
    }
}

impl<S> Default for LazyCompiler<S>
where
    S: PartialSource + Default,
{
    fn default() -> Self {
        Self {
            source: Default::default(),
        }
    }
}

impl<S> ::std::ops::Deref for LazyCompiler<S>
where
    S: PartialSource,
{
    type Target = S;

    fn deref(&self) -> &S {
        &self.source
    }
}

impl<S> ::std::ops::DerefMut for LazyCompiler<S>
where
    S: PartialSource,
{
    fn deref_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

impl<S> PartialCompiler for LazyCompiler<S>
where
    S: PartialSource + Send + Sync + 'static,
{
    fn compile(self, language: sync::Arc<Language>) -> Result<Box<dyn PartialStore + Send + Sync>> {
        let store = LazyStore {
            language,
            source: self.source,
            cache: sync::Mutex::new(Default::default()),
        };
        Ok(Box::new(store))
    }

    fn source(&self) -> &dyn PartialSource {
        &self.source
    }
}

struct LazyStore<S: PartialSource> {
    language: sync::Arc<Language>,
    source: S,
    cache: sync::Mutex<HashMap<String, Result<sync::Arc<dyn runtime::Renderable>>>>,
}

impl<S> LazyStore<S>
where
    S: PartialSource,
{
    fn try_get_or_create(&self, name: &str) -> Option<sync::Arc<dyn Renderable>> {
        let mut cache = self.cache.lock().expect("not to be poisoned and reused");
        if let Some(result) = cache.get(name) {
            result.as_ref().ok().cloned()
        } else {
            let s = self.source.try_get(name)?;
            let s = s.as_ref();
            let template = parser::parse(s, &self.language)
                .map(runtime::Template::new)
                .map(sync::Arc::new)
                .map(|t| t as sync::Arc<dyn Renderable>);
            cache.insert(name.to_string(), template.clone());
            template.ok()
        }
    }

    fn get_or_create(&self, name: &str) -> Result<sync::Arc<dyn Renderable>> {
        let mut cache = self.cache.lock().expect("not to be poisoned and reused");
        if let Some(result) = cache.get(name) {
            result.clone()
        } else {
            let s = self.source.get(name)?;
            let s = s.as_ref();
            let template = parser::parse(s, &self.language)
                .map(runtime::Template::new)
                .map(sync::Arc::new)
                .map(|t| t as sync::Arc<dyn Renderable>);
            cache.insert(name.to_string(), template.clone());
            template
        }
    }
}

impl<S> PartialStore for LazyStore<S>
where
    S: PartialSource,
{
    fn contains(&self, name: &str) -> bool {
        self.source.contains(name)
    }

    fn names(&self) -> Vec<&str> {
        self.source.names()
    }

    fn try_get(&self, name: &str) -> Option<sync::Arc<dyn Renderable>> {
        self.try_get_or_create(name)
    }

    fn get(&self, name: &str) -> Result<sync::Arc<dyn Renderable>> {
        self.get_or_create(name)
    }
}

impl<S> fmt::Debug for LazyStore<S>
where
    S: PartialSource,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use crate::partials::lazy;
    use crate::runtime::PartialStore;
    use crate::{partials, Language};
    use std::{borrow, sync};

    #[derive(Default, Debug, Clone, Copy)]
    struct TestSource;

    impl partials::PartialSource for TestSource {
        fn contains(&self, _name: &str) -> bool {
            true
        }

        fn names(&self) -> Vec<&str> {
            vec![]
        }

        fn try_get<'a>(&'a self, name: &str) -> Option<borrow::Cow<'a, str>> {
            match name {
                "example.txt" => Some("Hello Liquid!".into()),
                _ => None,
            }
        }
    }

    #[test]
    fn test_store_caches_get() {
        let options = Language::empty();
        let store = lazy::LazyStore {
            language: sync::Arc::new(options),
            source: TestSource,
            cache: sync::Mutex::new(Default::default()),
        };

        assert!(
            !store.cache.lock().unwrap().contains_key("example.txt"),
            "The store cache should not contain the key yet."
        );

        // Look up the partial, causing it to be cached
        let _ = store.get("example.txt").unwrap();

        assert!(
            store.cache.lock().unwrap().contains_key("example.txt"),
            "The store cache should now contain the key."
        );
    }

    #[test]
    fn test_store_caches_try_get() {
        let options = Language::empty();
        let store = lazy::LazyStore {
            language: sync::Arc::new(options),
            source: TestSource,
            cache: sync::Mutex::new(Default::default()),
        };

        assert!(
            !store.cache.lock().unwrap().contains_key("example.txt"),
            "The store cache should not contain the key yet."
        );

        // Look up the partial, causing it to be cached.
        let _ = store.try_get("example.txt").unwrap();

        assert!(
            store.cache.lock().unwrap().contains_key("example.txt"),
            "The store cache should now contain the key."
        );
    }
}
