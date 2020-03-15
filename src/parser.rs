use std::fs::File;
use std::io::prelude::Read;
use std::path;
use std::sync;

use liquid_core::compiler;
use liquid_core::error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use liquid_core::interpreter;

use super::Template;
use crate::reflection;
use liquid_core::partials;
#[cfg(feature = "stdlib")]
use liquid_lib::filters;
#[cfg(feature = "stdlib")]
use liquid_lib::tags;

/// Storage for partial-templates.
///
/// This is the recommended policy.  See `liquid::partials` for more options.
pub type Partials = partials::EagerCompiler<partials::InMemorySource>;

pub struct ParserBuilder<P = Partials>
where
    P: partials::PartialCompiler,
{
    blocks: compiler::PluginRegistry<Box<dyn compiler::ParseBlock>>,
    tags: compiler::PluginRegistry<Box<dyn compiler::ParseTag>>,
    filters: compiler::PluginRegistry<Box<dyn compiler::ParseFilter>>,
    partials: Option<P>,
}

impl ParserBuilder<Partials> {
    /// Create an empty Liquid parser
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "stdlib")]
    pub fn with_stdlib() -> Self {
        Self::new().stdlib()
    }
}

impl<P> ParserBuilder<P>
where
    P: partials::PartialCompiler,
{
    #[cfg(feature = "stdlib")]
    /// Create a Liquid parser with built-in Liquid features
    pub fn stdlib(self) -> Self {
        self.tag(tags::AssignTag)
            .tag(tags::BreakTag)
            .tag(tags::ContinueTag)
            .tag(tags::CycleTag)
            .tag(tags::IncludeTag)
            .tag(tags::IncrementTag)
            .tag(tags::DecrementTag)
            .block(tags::RawBlock)
            .block(tags::IfBlock)
            .block(tags::UnlessBlock)
            .block(tags::IfChangedBlock)
            .block(tags::ForBlock)
            .block(tags::TableRowBlock)
            .block(tags::CommentBlock)
            .block(tags::CaptureBlock)
            .block(tags::CaseBlock)
            .filter(filters::std::Append)
            .filter(filters::std::AtLeast)
            .filter(filters::std::AtMost)
            .filter(filters::std::Capitalize)
            .filter(filters::std::Ceil)
            .filter(filters::std::Compact)
            .filter(filters::std::Concat)
            .filter(filters::std::Date)
            .filter(filters::std::Default)
            .filter(filters::std::DividedBy)
            .filter(filters::std::Downcase)
            .filter(filters::std::Escape)
            .filter(filters::std::EscapeOnce)
            .filter(filters::std::First)
            .filter(filters::std::Floor)
            .filter(filters::std::Join)
            .filter(filters::std::Last)
            .filter(filters::std::Lstrip)
            .filter(filters::std::Map)
            .filter(filters::std::Minus)
            .filter(filters::std::Modulo)
            .filter(filters::std::NewlineToBr)
            .filter(filters::std::Plus)
            .filter(filters::std::Prepend)
            .filter(filters::std::Remove)
            .filter(filters::std::RemoveFirst)
            .filter(filters::std::Replace)
            .filter(filters::std::ReplaceFirst)
            .filter(filters::std::Reverse)
            .filter(filters::std::Round)
            .filter(filters::std::Rstrip)
            .filter(filters::std::Size)
            .filter(filters::std::Slice)
            .filter(filters::std::Sort)
            .filter(filters::std::SortNatural)
            .filter(filters::std::Split)
            .filter(filters::std::Strip)
            .filter(filters::std::StripHtml)
            .filter(filters::std::StripNewlines)
            .filter(filters::std::Times)
            .filter(filters::std::Truncate)
            .filter(filters::std::TruncateWords)
            .filter(filters::std::Uniq)
            .filter(filters::std::Upcase)
            .filter(filters::std::UrlDecode)
            .filter(filters::std::UrlEncode)
    }

    /// Inserts a new custom block into the parser
    pub fn block<B: Into<Box<dyn compiler::ParseBlock>>>(mut self, block: B) -> Self {
        let block = block.into();
        self.blocks
            .register(block.reflection().start_tag().to_owned(), block);
        self
    }

    /// Inserts a new custom tag into the parser
    pub fn tag<T: Into<Box<dyn compiler::ParseTag>>>(mut self, tag: T) -> Self {
        let tag = tag.into();
        self.tags.register(tag.reflection().tag().to_owned(), tag);
        self
    }

    /// Inserts a new custom filter into the parser
    pub fn filter<F: Into<Box<dyn compiler::ParseFilter>>>(mut self, filter: F) -> Self {
        let filter = filter.into();
        self.filters
            .register(filter.reflection().name().to_owned(), filter);
        self
    }

    /// Set which partial-templates will be available.
    pub fn partials<N: partials::PartialCompiler>(self, partials: N) -> ParserBuilder<N> {
        let Self {
            blocks,
            tags,
            filters,
            partials: _partials,
        } = self;
        ParserBuilder {
            blocks,
            tags,
            filters,
            partials: Some(partials),
        }
    }

    /// Create a parser
    pub fn build(self) -> Result<Parser> {
        let Self {
            blocks,
            tags,
            filters,
            partials,
        } = self;

        let mut options = compiler::Language::empty();
        options.blocks = blocks;
        options.tags = tags;
        options.filters = filters;
        let options = sync::Arc::new(options);
        let partials = partials
            .map(|p| p.compile(options.clone()))
            .map_or(Ok(None), |r| r.map(Some))?
            .map(|p| p.into());
        let p = Parser { options, partials };
        Ok(p)
    }
}

impl<P> Default for ParserBuilder<P>
where
    P: partials::PartialCompiler,
{
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            tags: Default::default(),
            filters: Default::default(),
            partials: Default::default(),
        }
    }
}

impl<P> reflection::ParserReflection for ParserBuilder<P>
where
    P: partials::PartialCompiler,
{
    fn blocks<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::BlockReflection> + 'r> {
        Box::new(self.blocks.plugins().map(|p| p.reflection()))
    }

    fn tags<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::TagReflection> + 'r> {
        Box::new(self.tags.plugins().map(|p| p.reflection()))
    }

    fn filters<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::FilterReflection> + 'r> {
        Box::new(self.filters.plugins().map(|p| p.reflection()))
    }

    fn partials<'r>(&'r self) -> Box<dyn Iterator<Item = &str> + 'r> {
        Box::new(
            self.partials
                .as_ref()
                .into_iter()
                .flat_map(|s| s.source().names()),
        )
    }
}

#[derive(Default, Clone)]
pub struct Parser {
    options: sync::Arc<compiler::Language>,
    partials: Option<sync::Arc<dyn interpreter::PartialStore + Send + Sync>>,
}

impl Parser {
    pub fn new() -> Self {
        Default::default()
    }

    /// Parses a liquid template, returning a Template object.
    /// # Examples
    ///
    /// ## Minimal Template
    ///
    /// ```
    /// let template = liquid::ParserBuilder::with_stdlib()
    ///     .build().unwrap()
    ///     .parse("Liquid!").unwrap();
    ///
    /// let globals = liquid::Object::new();
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid!".to_string());
    /// ```
    ///
    pub fn parse(&self, text: &str) -> Result<Template> {
        let template = compiler::parse(text, &self.options).map(interpreter::Template::new)?;
        Ok(Template {
            template,
            partials: self.partials.clone(),
        })
    }

    /// Parse a liquid template from a file, returning a `Result<Template, Error>`.
    /// # Examples
    ///
    /// ## Minimal Template
    ///
    /// `template.txt`:
    ///
    /// ```text
    /// "Liquid {{data}}"
    /// ```
    ///
    /// Your rust code:
    ///
    /// ```rust,no_run
    /// let template = liquid::ParserBuilder::with_stdlib()
    ///     .build().unwrap()
    ///     .parse_file("path/to/template.txt").unwrap();
    ///
    /// let globals = liquid::object!({
    ///     "data": 4f64,
    /// });
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid! 4\n".to_string());
    /// ```
    ///
    pub fn parse_file<P: AsRef<path::Path>>(&self, file: P) -> Result<Template> {
        self.parse_file_path(file.as_ref())
    }

    fn parse_file_path(&self, file: &path::Path) -> Result<Template> {
        let mut f = File::open(file)
            .replace("Cannot open file")
            .context_key("path")
            .value_with(|| file.to_string_lossy().into_owned().into())?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .replace("Cannot read file")
            .context_key("path")
            .value_with(|| file.to_string_lossy().into_owned().into())?;

        self.parse(&buf)
    }
}

impl reflection::ParserReflection for Parser {
    fn blocks<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::BlockReflection> + 'r> {
        Box::new(self.options.blocks.plugins().map(|p| p.reflection()))
    }

    fn tags<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::TagReflection> + 'r> {
        Box::new(self.options.tags.plugins().map(|p| p.reflection()))
    }

    fn filters<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::FilterReflection> + 'r> {
        Box::new(self.options.filters.plugins().map(|p| p.reflection()))
    }

    fn partials<'r>(&'r self) -> Box<dyn Iterator<Item = &str> + 'r> {
        Box::new(self.partials.as_ref().into_iter().flat_map(|s| s.names()))
    }
}
