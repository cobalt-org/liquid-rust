use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;
use std::path;
use std::sync;

use liquid_compiler as compiler;
use liquid_error::{Result, ResultLiquidChainExt, ResultLiquidExt};
use liquid_interpreter as interpreter;

use super::Template;
use filters;
use tags;

#[derive(Default, Clone)]
pub struct ParserBuilder {
    blocks: HashMap<&'static str, compiler::BoxedBlockParser>,
    tags: HashMap<&'static str, compiler::BoxedTagParser>,
    filters: HashMap<&'static str, interpreter::BoxedValueFilter>,
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
        self.filter("abs", filters::abs as interpreter::FnFilterValue)
            .filter("append", filters::append as interpreter::FnFilterValue)
            .filter("at_least", filters::at_least as interpreter::FnFilterValue)
            .filter("at_most", filters::at_most as interpreter::FnFilterValue)
            .filter(
                "capitalize",
                filters::capitalize as interpreter::FnFilterValue,
            ).filter("ceil", filters::ceil as interpreter::FnFilterValue)
            .filter("compact", filters::compact as interpreter::FnFilterValue)
            .filter("concat", filters::concat as interpreter::FnFilterValue)
            .filter("date", filters::date as interpreter::FnFilterValue)
            .filter("default", filters::default as interpreter::FnFilterValue)
            .filter(
                "divided_by",
                filters::divided_by as interpreter::FnFilterValue,
            ).filter("downcase", filters::downcase as interpreter::FnFilterValue)
            .filter("escape", filters::escape as interpreter::FnFilterValue)
            .filter(
                "escape_once",
                filters::escape_once as interpreter::FnFilterValue,
            ).filter("first", filters::first as interpreter::FnFilterValue)
            .filter("floor", filters::floor as interpreter::FnFilterValue)
            .filter("join", filters::join as interpreter::FnFilterValue)
            .filter("last", filters::last as interpreter::FnFilterValue)
            .filter("lstrip", filters::lstrip as interpreter::FnFilterValue)
            .filter("map", filters::map as interpreter::FnFilterValue)
            .filter("minus", filters::minus as interpreter::FnFilterValue)
            .filter("modulo", filters::modulo as interpreter::FnFilterValue)
            .filter(
                "newline_to_br",
                filters::newline_to_br as interpreter::FnFilterValue,
            ).filter("plus", filters::plus as interpreter::FnFilterValue)
            .filter("prepend", filters::prepend as interpreter::FnFilterValue)
            .filter("remove", filters::remove as interpreter::FnFilterValue)
            .filter(
                "remove_first",
                filters::remove_first as interpreter::FnFilterValue,
            ).filter("replace", filters::replace as interpreter::FnFilterValue)
            .filter(
                "replace_first",
                filters::replace_first as interpreter::FnFilterValue,
            ).filter("reverse", filters::reverse as interpreter::FnFilterValue)
            .filter("round", filters::round as interpreter::FnFilterValue)
            .filter("rstrip", filters::rstrip as interpreter::FnFilterValue)
            .filter("size", filters::size as interpreter::FnFilterValue)
            .filter("slice", filters::slice as interpreter::FnFilterValue)
            .filter("sort", filters::sort as interpreter::FnFilterValue)
            .filter(
                "sort_natural",
                filters::sort_natural as interpreter::FnFilterValue,
            ).filter("split", filters::split as interpreter::FnFilterValue)
            .filter("strip", filters::strip as interpreter::FnFilterValue)
            .filter(
                "strip_html",
                filters::strip_html as interpreter::FnFilterValue,
            ).filter(
                "strip_newlines",
                filters::strip_newlines as interpreter::FnFilterValue,
            ).filter("times", filters::times as interpreter::FnFilterValue)
            .filter("truncate", filters::truncate as interpreter::FnFilterValue)
            .filter(
                "truncatewords",
                filters::truncatewords as interpreter::FnFilterValue,
            ).filter("uniq", filters::uniq as interpreter::FnFilterValue)
            .filter("upcase", filters::upcase as interpreter::FnFilterValue)
            .filter(
                "url_decode",
                filters::url_decode as interpreter::FnFilterValue,
            ).filter(
                "url_encode",
                filters::url_encode as interpreter::FnFilterValue,
            )
    }

    /// Register non-standard filters
    #[cfg(not(feature = "extra-filters"))]
    pub fn extra_filters(self) -> Self {
        self
    }

    /// Register non-standard filters
    #[cfg(feature = "extra-filters")]
    pub fn extra_filters(self) -> Self {
        self.filter(
            "pluralize",
            filters::pluralize as interpreter::FnFilterValue,
        ).filter(
            "date_in_tz",
            filters::date_in_tz as interpreter::FnFilterValue,
        ).filter("push", filters::push as interpreter::FnFilterValue)
        .filter("pop", filters::pop as interpreter::FnFilterValue)
        .filter("unshift", filters::unshift as interpreter::FnFilterValue)
        .filter("shift", filters::shift as interpreter::FnFilterValue)
        .filter(
            "array_to_sentence_string",
            filters::array_to_sentence_string as interpreter::FnFilterValue,
        )
    }

    /// Inserts a new custom block into the parser
    pub fn block<B: Into<compiler::BoxedBlockParser>>(
        mut self,
        name: &'static str,
        block: B,
    ) -> Self {
        self.blocks.insert(name, block.into());
        self
    }

    /// Inserts a new custom tag into the parser
    pub fn tag<T: Into<compiler::BoxedTagParser>>(mut self, name: &'static str, tag: T) -> Self {
        self.tags.insert(name, tag.into());
        self
    }

    /// Inserts a new custom filter into the parser
    pub fn filter<F: Into<interpreter::BoxedValueFilter>>(
        mut self,
        name: &'static str,
        filter: F,
    ) -> Self {
        self.filters.insert(name, filter.into());
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
            include_source,
        };
        let filters = sync::Arc::new(filters);
        Parser { options, filters }
    }
}

#[derive(Default, Clone)]
pub struct Parser {
    options: compiler::LiquidOptions,
    filters: sync::Arc<HashMap<&'static str, interpreter::BoxedValueFilter>>,
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
        let filters = sync::Arc::clone(&self.filters);
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
            .context_with(|| ("path".into(), file.to_string_lossy().into()))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .chain("Cannot read file")
            .context_with(|| ("path".into(), file.to_string_lossy().into()))?;

        self.parse(&buf)
    }
}
