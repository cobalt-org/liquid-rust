use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::KString;
use liquid_core::parser::TryMatchToken;
use liquid_core::Expression;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::ValueView;
use liquid_core::{runtime::StackFrame, Runtime};
use liquid_core::{Error, Result};
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct IncludeTag;

impl IncludeTag {
    pub fn new() -> Self {
        Self
    }
}

impl TagReflection for IncludeTag {
    fn tag(&self) -> &'static str {
        "include"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for IncludeTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let name = arguments.expect_next("Identifier or literal expected.")?;

        // This may accept strange inputs such as `{% include 0 %}` or `{% include filterchain | filter:0 %}`.
        // Those inputs would fail anyway by there being not a path with those names so they are not a big concern.
        let name = match name.expect_identifier() {
            // Using `to_kstr()` on literals ensures `Strings` will have their quotes trimmed.
            TryMatchToken::Matches(name) => name.to_kstr().to_string(),
            TryMatchToken::Fails(name) => name.as_str().to_owned(),
        };

        let partial = Expression::with_literal(name);

        let mut vars: Vec<(KString, Expression)> = Vec::new();
        while let Ok(next) = arguments.expect_next("") {
            let id = next.expect_identifier().into_result()?.to_owned();

            arguments
                .expect_next("\"=\" expected.")?
                .expect_str("=")
                .into_result_custom_msg("expected \"=\" to be used for the assignment")?;

            vars.push((
                id.into(),
                arguments
                    .expect_next("expected value")?
                    .expect_value()
                    .into_result()?,
            ));
        }

        arguments.expect_nothing()?;

        Ok(Box::new(Include { partial, vars }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Include {
    partial: Expression,
    vars: Vec<(KString, Expression)>,
}

impl Renderable for Include {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let name = self.partial.evaluate(runtime)?.render().to_string();

        {
            let mut pass_through = std::collections::HashMap::<
                liquid_core::model::KStringRef<'_>,
                &dyn ValueView,
            >::new();
            let mut helper_vars = std::collections::HashMap::new();
            if !self.vars.is_empty() {
                for (id, val) in &self.vars {
                    let value = val
                        .try_evaluate(runtime)
                        .ok_or_else(|| Error::with_msg("failed to evaluate value"))?
                        .into_owned();

                    helper_vars.insert(id.as_ref(), value);
                }

                pass_through.insert("include".into(), &helper_vars);
            }

            let scope = StackFrame::new(runtime, &pass_through);
            let partial = scope
                .partials()
                .get(&name)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())?;

            partial
                .render_to(writer, &scope)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())
                .context_key_with(|| self.partial.to_string().into())
                .value_with(|| name.clone().into())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::borrow;

    use liquid_core::parser;
    use liquid_core::partials;
    use liquid_core::partials::PartialCompiler;
    use liquid_core::runtime;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::Value;
    use liquid_core::{Display_filter, Filter, FilterReflection, ParseFilter};

    use crate::stdlib;

    use super::*;

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
                "example.txt" => Some(r#"{{'whooo' | size}}{%comment%}What happens{%endcomment%} {%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}"#.into()),
                "example_var.txt" => Some(r#"{{include.example_var}}"#.into()),
                "example_multi_var.txt" => Some(r#"{{include.example_var}} {{include.example}}"#.into()),
                _ => None
            }
        }
    }

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("include".to_owned(), IncludeTag.into());
        options
            .blocks
            .register("comment".to_owned(), stdlib::CommentBlock.into());
        options
            .blocks
            .register("if".to_owned(), stdlib::IfBlock.into());
        options
    }

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(name = "size", description = "tests helper", parsed(SizeFilter))]
    pub(super) struct SizeFilterParser;

    #[derive(Debug, Default, Display_filter)]
    #[name = "size"]
    pub(super) struct SizeFilter;

    impl Filter for SizeFilter {
        fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
            if let Some(x) = input.as_scalar() {
                Ok(Value::scalar(x.to_kstr().len() as i64))
            } else if let Some(x) = input.as_array() {
                Ok(Value::scalar(x.size()))
            } else if let Some(x) = input.as_object() {
                Ok(Value::scalar(x.size()))
            } else {
                Ok(Value::scalar(0i64))
            }
        }
    }

    #[test]
    fn include_identifier() {
        let text = "{% include example.txt %}";
        let mut options = options();
        options
            .filters
            .register("size".to_owned(), Box::new(SizeFilterParser));
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let runtime = RuntimeBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        runtime.set_global("num".into(), Value::scalar(5f64));
        runtime.set_global("numTwo".into(), Value::scalar(10f64));
        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "5 wat wot");
    }

    #[test]
    fn include_variable() {
        let text = "{% include example_var.txt example_var=\"hello\" %}";
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let runtime = RuntimeBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn include_multiple_variable() {
        let text = "{% include example_multi_var.txt example_var=\"hello\" example=\"world\" %}";
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let runtime = RuntimeBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn no_file() {
        let text = "{% include 'file_does_not_exist.liquid' %}";
        let mut options = options();
        options
            .filters
            .register("size".to_owned(), Box::new(SizeFilterParser));
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let runtime = RuntimeBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        runtime.set_global("num".into(), Value::scalar(5f64));
        runtime.set_global("numTwo".into(), Value::scalar(10f64));
        let output = template.render(&runtime);
        assert!(output.is_err());
    }
}
