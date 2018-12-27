use std::fmt;
use std::sync;

use liquid_compiler;
use liquid_compiler::Language;
use liquid_error::Result;
use liquid_interpreter;
use liquid_interpreter::PartialStore;
use liquid_interpreter::Renderable;

use super::PartialCompiler;
use super::PartialSource;

/// An on-demand compiler for `PartialSource`.
///
/// This is unlikely the `PartialCompiler` you want.  This best serves as an example.
///
/// This would be useful in cases where:
/// - Most partial-templates aren't used
/// - Of the used partial-templates, they are generally used once.
#[derive(Debug)]
pub struct OnDemandCompiler<S: PartialSource> {
    source: S,
}

impl<S> OnDemandCompiler<S>
where
    S: PartialSource,
{
    /// Create an on-demand compiler for `PartialSource`.
    pub fn new(source: S) -> Self {
        OnDemandCompiler { source }
    }
}

impl<S> OnDemandCompiler<S>
where
    S: PartialSource + Default,
{
    /// Create an empty compiler for `PartialSource`.
    pub fn empty() -> Self {
        Default::default()
    }
}

impl<S> Default for OnDemandCompiler<S>
where
    S: PartialSource + Default,
{
    fn default() -> Self {
        Self {
            source: Default::default(),
        }
    }
}

impl<S> ::std::ops::Deref for OnDemandCompiler<S>
where
    S: PartialSource,
{
    type Target = S;

    fn deref(&self) -> &S {
        &self.source
    }
}

impl<S> ::std::ops::DerefMut for OnDemandCompiler<S>
where
    S: PartialSource,
{
    fn deref_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

impl<S> PartialCompiler for OnDemandCompiler<S>
where
    S: PartialSource + Send + Sync + 'static,
{
    fn compile(self, language: sync::Arc<Language>) -> Result<Box<PartialStore + Send + Sync>> {
        let store = OnDemandStore {
            language,
            source: self.source,
        };
        Ok(Box::new(store))
    }
}

struct OnDemandStore<S: PartialSource> {
    language: sync::Arc<Language>,
    source: S,
}

impl<S> PartialStore for OnDemandStore<S>
where
    S: PartialSource,
{
    fn contains(&self, name: &str) -> bool {
        self.source.contains(name)
    }

    fn names(&self) -> Vec<&str> {
        self.source.names()
    }

    fn try_get(&self, name: &str) -> Option<sync::Arc<Renderable>> {
        let s = self.source.try_get(name)?;
        let s = s.as_ref();
        let template = liquid_compiler::parse(s, &self.language)
            .map(liquid_interpreter::Template::new)
            .map(sync::Arc::new)
            .ok()?;
        Some(template)
    }

    fn get(&self, name: &str) -> Result<sync::Arc<Renderable>> {
        let s = self.source.get(name)?;
        let s = s.as_ref();
        let template = liquid_compiler::parse(s, &self.language)
            .map(liquid_interpreter::Template::new)
            .map(sync::Arc::new)?;
        Ok(template)
    }
}

impl<S> fmt::Debug for OnDemandStore<S>
where
    S: PartialSource,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.source.fmt(f)
    }
}
