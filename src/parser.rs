use std::fs::File;
use std::io::prelude::Read;
use std::path;
use std::sync;

use liquid_compiler as compiler;
use liquid_error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};
use liquid_interpreter as interpreter;

use super::Template;
use filters;
use partials;
use tags;

/// Storage for partial-templates.
///
/// This is the recommended policy.  See `liquid::partials` for more options.
pub type Partials = partials::EagerCompiler<partials::InMemorySource>;

pub struct ParserBuilder<P = Partials>
where
    P: partials::PartialCompiler,
{
    blocks: compiler::PluginRegistry<compiler::BoxedBlockParser>,
    tags: compiler::PluginRegistry<compiler::BoxedTagParser>,
    filters: compiler::PluginRegistry<Box<compiler::ParseFilter>>,
    partials: Option<P>,
}

impl ParserBuilder<Partials> {
    /// Create an empty Liquid parser
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_liquid() -> Self {
        Self::new().liquid()
    }
}

impl<P> ParserBuilder<P>
where
    P: partials::PartialCompiler,
{
    /// Create a Liquid parser with built-in Liquid features
    pub fn liquid(self) -> Self {
        self.liquid_tags().liquid_blocks().liquid_filters()
    }

    /// Register built-in Liquid tags
    pub fn liquid_tags(self) -> Self {
        self.tag("assign", tags::assign_tag as compiler::FnParseTag)
            .tag("break", tags::break_tag as compiler::FnParseTag)
            .tag("continue", tags::continue_tag as compiler::FnParseTag)
            .tag("cycle", tags::cycle_tag as compiler::FnParseTag)
            .tag("include", tags::include_tag as compiler::FnParseTag)
            .tag("increment", tags::increment_tag as compiler::FnParseTag)
            .tag("decrement", tags::decrement_tag as compiler::FnParseTag)
    }

    /// Register built-in Liquid blocks
    pub fn liquid_blocks(self) -> Self {
        self.block("raw", tags::raw_block as compiler::FnParseBlock)
            .block("if", tags::if_block as compiler::FnParseBlock)
            .block("unless", tags::unless_block as compiler::FnParseBlock)
            .block("ifchanged", tags::ifchanged_block as compiler::FnParseBlock)
            .block("for", tags::for_block as compiler::FnParseBlock)
            .block("tablerow", tags::tablerow_block as compiler::FnParseBlock)
            .block("comment", tags::comment_block as compiler::FnParseBlock)
            .block("capture", tags::capture_block as compiler::FnParseBlock)
            .block("case", tags::case_block as compiler::FnParseBlock)
    }

    /// Register built-in Liquid filters
    pub fn liquid_filters(self) -> Self {
        self.filter(filters::std::Abs)
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

    /// Register non-standard filters
    #[cfg(not(feature = "extra-filters"))]
    pub fn extra_filters(self) -> Self {
        self
    }

    /// Register non-standard filters
    #[cfg(feature = "extra-filters")]
    pub fn extra_filters(self) -> Self {
        self.filter(filters::extra::DateInTz)
            .filter(filters::extra::Pluralize)
    }

    /// Register non-standard filters
    #[cfg(not(feature = "jekyll-filters"))]
    pub fn jekyll_filters(self) -> Self {
        self
    }

    /// Register non-standard filters
    #[cfg(feature = "jekyll-filters")]
    pub fn jekyll_filters(self) -> Self {
        self.filter(filters::jekyll::Slugify)
            .filter(filters::jekyll::Pop)
            .filter(filters::jekyll::Push)
            .filter(filters::jekyll::Shift)
            .filter(filters::jekyll::Unshift)
            .filter(filters::jekyll::ArrayToSentenceString)
    }

    /// Inserts a new custom block into the parser
    pub fn block<B: Into<compiler::BoxedBlockParser>>(
        mut self,
        name: &'static str,
        block: B,
    ) -> Self {
        self.blocks.register(name, block.into());
        self
    }

    /// Inserts a new custom tag into the parser
    pub fn tag<T: Into<compiler::BoxedTagParser>>(mut self, name: &'static str, tag: T) -> Self {
        self.tags.register(name, tag.into());
        self
    }

    /// Inserts a new custom filter into the parser
    pub fn filter<F: Into<Box<compiler::ParseFilter>>>(mut self, filter: F) -> Self {
        let filter = filter.into();
        self.filters
            .register(compiler::FilterReflection::name(&*filter), filter);
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

#[derive(Default, Clone)]
pub struct Parser {
    options: sync::Arc<compiler::Language>,
    partials: Option<sync::Arc<interpreter::PartialStore + Send + Sync>>,
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
    /// let template = liquid::ParserBuilder::with_liquid()
    ///     .build().unwrap()
    ///     .parse("Liquid!").unwrap();
    ///
    /// let globals = liquid::value::Object::new();
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
    /// let template = liquid::ParserBuilder::with_liquid()
    ///     .build().unwrap()
    ///     .parse_file("path/to/template.txt").unwrap();
    ///
    /// let mut globals = liquid::value::Object::new();
    /// globals.insert("data".into(), liquid::value::Value::scalar(4f64));
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid! 4\n".to_string());
    /// ```
    ///
    pub fn parse_file<P: AsRef<path::Path>>(self, file: P) -> Result<Template> {
        self.parse_file_path(file.as_ref())
    }

    fn parse_file_path(self, file: &path::Path) -> Result<Template> {
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
