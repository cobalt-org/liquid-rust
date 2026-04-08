use liquid_core::Language;
use liquid_core::{ParseTag, Result, TagReflection, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct EchoTag;

impl EchoTag {
    pub fn new() -> Self {
        Self
    }
}

impl TagReflection for EchoTag {
    fn tag(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for EchoTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        options: &Language,
    ) -> Result<Box<dyn liquid_core::Renderable>> {
        let chain = arguments
            .expect_next("FilterChain expected.")?
            .parse_as_filter_chain(options)?;
        arguments.expect_nothing()?;
        Ok(Box::new(chain))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::parser;
    use liquid_core::parser::{FilterArguments, ParameterReflection};
    use liquid_core::{Error, Filter, FilterReflection, ParseFilter};

    #[derive(Clone)]
    struct FailingFilterParser;

    impl FilterReflection for FailingFilterParser {
        fn name(&self) -> &str {
            "fail_parse"
        }

        fn description(&self) -> &str {
            "test helper"
        }

        fn positional_parameters(&self) -> &'static [ParameterReflection] {
            &[]
        }

        fn keyword_parameters(&self) -> &'static [ParameterReflection] {
            &[]
        }
    }

    impl ParseFilter for FailingFilterParser {
        fn parse(&self, _arguments: FilterArguments<'_>) -> Result<Box<dyn Filter>> {
            Err(Error::with_msg("specific filter parse failure"))
        }

        fn reflection(&self) -> &dyn FilterReflection {
            self
        }
    }

    fn options() -> Language {
        let mut options = Language::default();
        options.tags.register("echo".to_owned(), EchoTag.into());
        std::sync::Arc::get_mut(&mut options.filters)
            .expect("default filter registry is uniquely owned")
            .register("fail_parse".to_owned(), Box::new(FailingFilterParser));
        options
    }

    #[test]
    fn parse_preserves_filter_chain_parser_errors() {
        let error = parser::parse("{% echo value | fail_parse %}", &options()).unwrap_err();
        let error = error.to_string();

        assert!(error.contains("specific filter parse failure"));
        assert!(!error.contains("FilterChain expected"));
    }
}
