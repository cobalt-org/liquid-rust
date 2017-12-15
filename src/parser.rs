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
    blocks: HashMap<String, Box<syntax::ParseBlock>>,
    tags: HashMap<String, Box<syntax::ParseTag>>,
    filters: HashMap<String, Box<syntax::Filter>>,
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
        self.tag("assign",
                 Box::new(syntax::FnTagParser::new(tags::assign_tag)))
            .tag("break", Box::new(syntax::FnTagParser::new(tags::break_tag)))
            .tag("continue",
                 Box::new(syntax::FnTagParser::new(tags::continue_tag)))
            .tag("cycle", Box::new(syntax::FnTagParser::new(tags::cycle_tag)))
            .tag("include",
                 Box::new(syntax::FnTagParser::new(tags::include_tag)))
    }

    /// Register built-in Liquid blocks
    pub fn liquid_blocks(self) -> Self {
        self.block("raw", Box::new(syntax::FnBlockParser::new(tags::raw_block)))
            .block("if", Box::new(syntax::FnBlockParser::new(tags::if_block)))
            .block("unless",
                   Box::new(syntax::FnBlockParser::new(tags::unless_block)))
            .block("for", Box::new(syntax::FnBlockParser::new(tags::for_block)))
            .block("comment",
                   Box::new(syntax::FnBlockParser::new(tags::comment_block)))
            .block("capture",
                   Box::new(syntax::FnBlockParser::new(tags::capture_block)))
            .block("case",
                   Box::new(syntax::FnBlockParser::new(tags::case_block)))
    }

    /// Register built-in Liquid filters
    pub fn liquid_filters(self) -> Self {
        self.filter("abs", Box::new(filters::abs))
            .filter("append", Box::new(filters::append))
            .filter("capitalize", Box::new(filters::capitalize))
            .filter("ceil", Box::new(filters::ceil))
            .filter("compact", Box::new(filters::compact))
            .filter("concat", Box::new(filters::concat))
            .filter("date", Box::new(filters::date))
            .filter("default", Box::new(filters::default))
            .filter("divided_by", Box::new(filters::divided_by))
            .filter("downcase", Box::new(filters::downcase))
            .filter("escape", Box::new(filters::escape))
            .filter("escape_once", Box::new(filters::escape_once))
            .filter("first", Box::new(filters::first))
            .filter("floor", Box::new(filters::floor))
            .filter("join", Box::new(filters::join))
            .filter("last", Box::new(filters::last))
            .filter("lstrip", Box::new(filters::lstrip))
            .filter("map", Box::new(filters::map))
            .filter("minus", Box::new(filters::minus))
            .filter("modulo", Box::new(filters::modulo))
            .filter("newline_to_br", Box::new(filters::newline_to_br))
            .filter("plus", Box::new(filters::plus))
            .filter("prepend", Box::new(filters::prepend))
            .filter("remove", Box::new(filters::remove))
            .filter("remove_first", Box::new(filters::remove_first))
            .filter("replace", Box::new(filters::replace))
            .filter("replace_first", Box::new(filters::replace_first))
            .filter("reverse", Box::new(filters::reverse))
            .filter("round", Box::new(filters::round))
            .filter("rstrip", Box::new(filters::rstrip))
            .filter("size", Box::new(filters::size))
            .filter("slice", Box::new(filters::slice))
            .filter("sort", Box::new(filters::sort))
            .filter("sort_natural", Box::new(filters::sort_natural))
            .filter("split", Box::new(filters::split))
            .filter("strip", Box::new(filters::strip))
            .filter("strip_html", Box::new(filters::strip_html))
            .filter("strip_newlines", Box::new(filters::strip_newlines))
            .filter("times", Box::new(filters::times))
            .filter("truncate", Box::new(filters::truncate))
            .filter("truncatewords", Box::new(filters::truncatewords))
            .filter("uniq", Box::new(filters::uniq))
            .filter("upcase", Box::new(filters::upcase))
            .filter("url_decode", Box::new(filters::url_decode))
            .filter("url_encode", Box::new(filters::url_encode))
    }

    #[cfg(not(feature = "extra-filters"))]
    pub fn extra_filters(self) -> Self {
        self
    }

    #[cfg(feature = "extra-filters")]
    pub fn extra_filters(self) -> Self {
        self.filter("pluralize", Box::new(filters::pluralize))
            .filter("date_in_tz", Box::new(filters::date_in_tz))
    }

    /// Inserts a new custom block into the parser
    pub fn block(mut self, name: &str, block: Box<syntax::ParseBlock>) -> Self {
        self.blocks.insert(name.to_owned(), block);
        self
    }

    /// Inserts a new custom tag into the parser
    pub fn tag(mut self, name: &str, tag: Box<syntax::ParseTag>) -> Self {
        self.tags.insert(name.to_owned(), tag);
        self
    }

    /// Inserts a new custom filter into the parser
    pub fn filter(mut self, name: &str, filter: Box<syntax::Filter>) -> Self {
        self.filters.insert(name.to_owned(), filter);
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
    filters: HashMap<String, Box<syntax::Filter>>,
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
    pub fn parse(self, text: &str) -> Result<Template> {
        let tokens = syntax::tokenize(text)?;
        let template = syntax::parse(&tokens, &self.options)
            .map(syntax::Template::new)?;
        let filters = self.filters;
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
