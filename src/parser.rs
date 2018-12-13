use std::fs::File;
use std::io::prelude::Read;
use std::path;

use liquid_compiler as compiler;
use liquid_error::{Result, ResultLiquidChainExt, ResultLiquidExt};
use liquid_interpreter as interpreter;

use super::Template;
use filters;
use tags;

#[derive(Default, Clone)]
pub struct ParserBuilder {
    blocks: compiler::PluginRegistry<compiler::BoxedBlockParser>,
    tags: compiler::PluginRegistry<compiler::BoxedTagParser>,
    filters: compiler::PluginRegistry<compiler::BoxedValueFilter>,
    include_source: Option<Box<compiler::Include>>,
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
        self.filter("abs", filters::abs as compiler::FnFilterValue)
            .filter("append", filters::append as compiler::FnFilterValue)
            .filter("at_least", filters::at_least as compiler::FnFilterValue)
            .filter("at_most", filters::at_most as compiler::FnFilterValue)
            .filter("capitalize", filters::capitalize as compiler::FnFilterValue)
            .filter("ceil", filters::ceil as compiler::FnFilterValue)
            .filter("compact", filters::compact as compiler::FnFilterValue)
            .filter("concat", filters::concat as compiler::FnFilterValue)
            .filter("date", filters::date as compiler::FnFilterValue)
            .filter("default", filters::default as compiler::FnFilterValue)
            .filter("divided_by", filters::divided_by as compiler::FnFilterValue)
            .filter("downcase", filters::downcase as compiler::FnFilterValue)
            .filter("escape", filters::escape as compiler::FnFilterValue)
            .filter(
                "escape_once",
                filters::escape_once as compiler::FnFilterValue,
            )
            .filter("first", filters::first as compiler::FnFilterValue)
            .filter("floor", filters::floor as compiler::FnFilterValue)
            .filter("join", filters::join as compiler::FnFilterValue)
            .filter("last", filters::last as compiler::FnFilterValue)
            .filter("lstrip", filters::lstrip as compiler::FnFilterValue)
            .filter("map", filters::map as compiler::FnFilterValue)
            .filter("minus", filters::minus as compiler::FnFilterValue)
            .filter("modulo", filters::modulo as compiler::FnFilterValue)
            .filter(
                "newline_to_br",
                filters::newline_to_br as compiler::FnFilterValue,
            )
            .filter("plus", filters::plus as compiler::FnFilterValue)
            .filter("prepend", filters::prepend as compiler::FnFilterValue)
            .filter("remove", filters::remove as compiler::FnFilterValue)
            .filter(
                "remove_first",
                filters::remove_first as compiler::FnFilterValue,
            )
            .filter("replace", filters::replace as compiler::FnFilterValue)
            .filter(
                "replace_first",
                filters::replace_first as compiler::FnFilterValue,
            )
            .filter("reverse", filters::reverse as compiler::FnFilterValue)
            .filter("round", filters::round as compiler::FnFilterValue)
            .filter("rstrip", filters::rstrip as compiler::FnFilterValue)
            .filter("size", filters::size as compiler::FnFilterValue)
            .filter("slice", filters::slice as compiler::FnFilterValue)
            .filter("sort", filters::sort as compiler::FnFilterValue)
            .filter(
                "sort_natural",
                filters::sort_natural as compiler::FnFilterValue,
            )
            .filter("split", filters::split as compiler::FnFilterValue)
            .filter("strip", filters::strip as compiler::FnFilterValue)
            .filter("strip_html", filters::strip_html as compiler::FnFilterValue)
            .filter(
                "strip_newlines",
                filters::strip_newlines as compiler::FnFilterValue,
            )
            .filter("times", filters::times as compiler::FnFilterValue)
            .filter("truncate", filters::truncate as compiler::FnFilterValue)
            .filter(
                "truncatewords",
                filters::truncatewords as compiler::FnFilterValue,
            )
            .filter("uniq", filters::uniq as compiler::FnFilterValue)
            .filter("upcase", filters::upcase as compiler::FnFilterValue)
            .filter("url_decode", filters::url_decode as compiler::FnFilterValue)
            .filter("url_encode", filters::url_encode as compiler::FnFilterValue)
    }

    /// Register non-standard filters
    #[cfg(not(feature = "extra-filters"))]
    pub fn extra_filters(self) -> Self {
        self
    }

    /// Register non-standard filters
    #[cfg(feature = "extra-filters")]
    pub fn extra_filters(self) -> Self {
        self.filter("pluralize", filters::pluralize as compiler::FnFilterValue)
            .filter("date_in_tz", filters::date_in_tz as compiler::FnFilterValue)
            .filter("push", filters::push as compiler::FnFilterValue)
            .filter("pop", filters::pop as compiler::FnFilterValue)
            .filter("unshift", filters::unshift as compiler::FnFilterValue)
            .filter("shift", filters::shift as compiler::FnFilterValue)
            .filter(
                "array_to_sentence_string",
                filters::array_to_sentence_string as compiler::FnFilterValue,
            )
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
    pub fn filter<F: Into<compiler::BoxedValueFilter>>(
        mut self,
        name: &'static str,
        filter: F,
    ) -> Self {
        self.filters.register(name, filter.into());
        self
    }

    /// Define the source for includes
    pub fn include_source(mut self, includes: Box<compiler::Include>) -> Self {
        self.include_source = Some(includes);
        self
    }

    /// Create a parser
    pub fn build(self) -> Parser {
        let Self {
            blocks,
            tags,
            filters,
            include_source,
        } = self;
        let include_source =
            include_source.unwrap_or_else(|| Box::new(compiler::NullInclude::new()));

        let options = compiler::LiquidOptions {
            blocks,
            tags,
            filters,
            include_source,
        };
        Parser { options }
    }
}

#[derive(Default, Clone)]
pub struct Parser {
    options: compiler::LiquidOptions,
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
    /// let globals = liquid::value::Object::new();
    /// let output = template.render(&globals).unwrap();
    /// assert_eq!(output, "Liquid!".to_string());
    /// ```
    ///
    pub fn parse(&self, text: &str) -> Result<Template> {
        let template = compiler::parse(text, &self.options).map(interpreter::Template::new)?;
        Ok(Template { template })
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
            .chain("Cannot open file")
            .context_key("path")
            .value_with(|| file.to_string_lossy().into_owned().into())?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .chain("Cannot read file")
            .context_key("path")
            .value_with(|| file.to_string_lossy().into_owned().into())?;

        self.parse(&buf)
    }
}
