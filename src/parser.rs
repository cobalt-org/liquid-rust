use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::path;

use error::Result;
use tags;
use filters;
use syntax;
use super::Template;

#[derive(Default)]
pub struct ParserBuilder {
    blocks: HashMap<String, syntax::BoxedBlockParser>,
    tags: HashMap<String, syntax::BoxedTagParser>,
    filters: HashMap<String, syntax::BoxedValueFilter>,
    include_source: Option<Box<syntax::Include>>,
}

impl ParserBuilder {
    /// Create an empty Liquid parser
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a Liquid parser with built-in Liquid features
    pub fn with_liquid() -> Self {
        Self::default()
            .liquid_tags()
            .liquid_blocks()
            .liquid_filters()
    }

    /// Register built-in Liquid tags
    pub fn liquid_tags(self) -> Self {
        self.tag("assign", tags::assign_tag as syntax::FnParseTag)
            .tag("break", tags::break_tag as syntax::FnParseTag)
            .tag("continue", tags::continue_tag as syntax::FnParseTag)
            .tag("cycle", tags::cycle_tag as syntax::FnParseTag)
            .tag("include", tags::include_tag as syntax::FnParseTag)
    }

    /// Register built-in Liquid blocks
    pub fn liquid_blocks(self) -> Self {
        self.block("raw", tags::raw_block as syntax::FnParseBlock)
            .block("if", tags::if_block as syntax::FnParseBlock)
            .block("unless", tags::unless_block as syntax::FnParseBlock)
            .block("for", tags::for_block as syntax::FnParseBlock)
            .block("comment", tags::comment_block as syntax::FnParseBlock)
            .block("capture", tags::capture_block as syntax::FnParseBlock)
            .block("case", tags::case_block as syntax::FnParseBlock)
    }

    /// Register built-in Liquid filters
    pub fn liquid_filters(self) -> Self {
        self.filter("abs", filters::abs as syntax::FnFilterValue)
            .filter("append", filters::append as syntax::FnFilterValue)
            .filter("capitalize", filters::capitalize as syntax::FnFilterValue)
            .filter("ceil", filters::ceil as syntax::FnFilterValue)
            .filter("compact", filters::compact as syntax::FnFilterValue)
            .filter("concat", filters::concat as syntax::FnFilterValue)
            .filter("date", filters::date as syntax::FnFilterValue)
            .filter("default", filters::default as syntax::FnFilterValue)
            .filter("divided_by", filters::divided_by as syntax::FnFilterValue)
            .filter("downcase", filters::downcase as syntax::FnFilterValue)
            .filter("escape", filters::escape as syntax::FnFilterValue)
            .filter("escape_once", filters::escape_once as syntax::FnFilterValue)
            .filter("first", filters::first as syntax::FnFilterValue)
            .filter("floor", filters::floor as syntax::FnFilterValue)
            .filter("join", filters::join as syntax::FnFilterValue)
            .filter("last", filters::last as syntax::FnFilterValue)
            .filter("lstrip", filters::lstrip as syntax::FnFilterValue)
            .filter("map", filters::map as syntax::FnFilterValue)
            .filter("minus", filters::minus as syntax::FnFilterValue)
            .filter("modulo", filters::modulo as syntax::FnFilterValue)
            .filter("newline_to_br",
                    filters::newline_to_br as syntax::FnFilterValue)
            .filter("plus", filters::plus as syntax::FnFilterValue)
            .filter("prepend", filters::prepend as syntax::FnFilterValue)
            .filter("remove", filters::remove as syntax::FnFilterValue)
            .filter("remove_first",
                    filters::remove_first as syntax::FnFilterValue)
            .filter("replace", filters::replace as syntax::FnFilterValue)
            .filter("replace_first",
                    filters::replace_first as syntax::FnFilterValue)
            .filter("reverse", filters::reverse as syntax::FnFilterValue)
            .filter("round", filters::round as syntax::FnFilterValue)
            .filter("rstrip", filters::rstrip as syntax::FnFilterValue)
            .filter("size", filters::size as syntax::FnFilterValue)
            .filter("slice", filters::slice as syntax::FnFilterValue)
            .filter("sort", filters::sort as syntax::FnFilterValue)
            .filter("sort_natural",
                    filters::sort_natural as syntax::FnFilterValue)
            .filter("split", filters::split as syntax::FnFilterValue)
            .filter("strip", filters::strip as syntax::FnFilterValue)
            .filter("strip_html", filters::strip_html as syntax::FnFilterValue)
            .filter("strip_newlines",
                    filters::strip_newlines as syntax::FnFilterValue)
            .filter("times", filters::times as syntax::FnFilterValue)
            .filter("truncate", filters::truncate as syntax::FnFilterValue)
            .filter("truncatewords",
                    filters::truncatewords as syntax::FnFilterValue)
            .filter("uniq", filters::uniq as syntax::FnFilterValue)
            .filter("upcase", filters::upcase as syntax::FnFilterValue)
            .filter("url_decode", filters::url_decode as syntax::FnFilterValue)
            .filter("url_encode", filters::url_encode as syntax::FnFilterValue)
    }

    #[cfg(not(feature = "extra-filters"))]
    pub fn extra_filters(self) -> Self {
        self
    }

    #[cfg(feature = "extra-filters")]
    pub fn extra_filters(self) -> Self {
        self.filter("pluralize", filters::pluralize as syntax::FnFilterValue)
            .filter("date_in_tz", filters::date_in_tz as syntax::FnFilterValue)
    }

    /// Inserts a new custom block into the parser
    pub fn block<B: Into<syntax::BoxedBlockParser>>(mut self, name: &str, block: B) -> Self {
        self.blocks.insert(name.to_owned(), block.into());
        self
    }

    /// Inserts a new custom tag into the parser
    pub fn tag<T: Into<syntax::BoxedTagParser>>(mut self, name: &str, tag: T) -> Self {
        self.tags.insert(name.to_owned(), tag.into());
        self
    }

    /// Inserts a new custom filter into the parser
    pub fn filter<F: Into<syntax::BoxedValueFilter>>(mut self, name: &str, filter: F) -> Self {
        self.filters.insert(name.to_owned(), filter.into());
        self
    }

    /// Define the source for includes
    pub fn include_source(mut self, includes: Box<syntax::Include>) -> Self {
        self.include_source = Some(includes);
        self
    }

    pub fn build(self) -> Parser {
        let Self {
            blocks,
            tags,
            filters,
            include_source,
        } = self;
        let include_source = include_source.unwrap_or_else(|| Box::new(syntax::NullInclude::new()));

        let options = syntax::LiquidOptions {
            blocks,
            tags,
            include_source,
        };
        Parser { options, filters }
    }
}

#[derive(Default)]
pub struct Parser {
    options: syntax::LiquidOptions,
    filters: HashMap<String, syntax::BoxedValueFilter>,
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
    ///     .build()
    ///     .parse("Liquid!").unwrap();
    ///
    /// let globals = liquid::Object::new();
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid!".to_string());
    /// ```
    ///
    pub fn parse(&self, text: &str) -> Result<Template> {
        let tokens = syntax::tokenize(text)?;
        let template = syntax::parse(&tokens, &self.options)
            .map(syntax::Template::new)?;
        let filters = self.filters.clone();
        Ok(Template { template, filters })
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
    ///     .build()
    ///     .parse_file("path/to/template.txt").unwrap();
    ///
    /// let mut globals = liquid::Object::new();
    /// globals.insert("data".to_owned(), liquid::Value::Num(4f32));
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid! 4\n".to_string());
    /// ```
    ///
    pub fn parse_file<P: AsRef<path::Path>>(self, file: P) -> Result<Template> {
        self.parse_file_path(file.as_ref())
    }

    fn parse_file_path(self, file: &path::Path) -> Result<Template> {
        let mut f = File::open(file)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;

        self.parse(&buf)
    }
}
