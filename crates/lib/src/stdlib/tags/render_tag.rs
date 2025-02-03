use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::KString;
use liquid_core::parser::TryMatchToken;
use liquid_core::runtime::GlobalFrame;
use liquid_core::runtime::Interrupt;
use liquid_core::runtime::InterruptRegister;
use liquid_core::runtime::SandboxedStackFrame;
use liquid_core::Expression;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Runtime;
use liquid_core::ValueView;
use liquid_core::{Error, Result};
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

use crate::stdlib::ForloopObject;
use crate::stdlib::RangeExpression;

#[derive(Copy, Clone, Debug, Default)]
pub struct RenderTag;

impl RenderTag {
    pub fn new() -> Self {
        Self
    }
}

impl TagReflection for RenderTag {
    fn tag(&self) -> &str {
        "render"
    }

    fn description(&self) -> &str {
        "insert the rendered content of another template in this template"
    }
}

impl ParseTag for RenderTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let partial = arguments.expect_next("Identifier or literal expected.")?;

        let partial = partial.expect_value().into_result()?;

        let mut token = arguments.next();
        let mut vars: Vec<(KString, Expression)> = Vec::new();
        let mut for_ = None;
        match token.as_ref().map(|t| t.as_str()) {
            Some("with") => {
                let val = arguments
                    .expect_next("expected value")?
                    .expect_value()
                    .into_result()?;

                arguments
                    .expect_next("\"as\" expected.")?
                    .expect_str("as")
                    .into_result_custom_msg("expected \"as\" to be used for the assignment")?;

                vars.push((
                    arguments
                        .expect_next("expected identifier")?
                        .expect_identifier()
                        .into_result()?
                        .to_owned()
                        .into(),
                    val,
                ));
                token = arguments.next();
            }
            Some("for") => {
                let range = arguments.expect_next("Array or range expected.")?;
                let range = match range.expect_value() {
                    TryMatchToken::Matches(array) => RangeExpression::Array(array),
                    TryMatchToken::Fails(range) => match range.expect_range() {
                        TryMatchToken::Matches((start, stop)) => {
                            RangeExpression::Counted(start, stop)
                        }
                        TryMatchToken::Fails(range) => return range.raise_error().into_err(),
                    },
                };

                arguments
                    .expect_next("\"as\" expected")?
                    .expect_str("as")
                    .into_result_custom_msg("\"as\" expected")?;

                let var_name = arguments
                    .expect_next("Identifier expected.")?
                    .expect_identifier()
                    .into_result()?
                    .to_owned()
                    .into();

                token = arguments.next();
                for_ = Some((range, var_name));
            }
            _ => {}
        };

        while let Some(t) = token {
            t.expect_str(",")
                .into_result_custom_msg("`,` is needed to separate variables")?;
            token = arguments.next();
            let Some(t) = token else {
                break;
            };

            let id = t.expect_identifier().into_result()?.to_owned();

            arguments
                .expect_next("\":\" expected.")?
                .expect_str(":")
                .into_result_custom_msg("expected \":\" to be used for the assignment")?;

            vars.push((
                id.into(),
                arguments
                    .expect_next("expected value")?
                    .expect_value()
                    .into_result()?,
            ));

            token = arguments.next();
        }

        arguments.expect_nothing()?;

        Ok(Box::new(Render {
            partial,
            for_,
            vars,
        }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Render {
    partial: Expression,
    for_: Option<(RangeExpression, KString)>,
    vars: Vec<(KString, Expression)>,
}

impl Renderable for Render {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let value = self.partial.evaluate(runtime)?;
        if !value.is_scalar() {
            return Error::with_msg("Can only `include` strings")
                .context("partial", format!("{}", value.source()))
                .into_err();
        }
        let name = value.to_kstr().into_owned();

        if let Some((range, var_name)) = &self.for_ {
            let range = range
                .evaluate(runtime)
                .trace_with(|| format!("{{% render {} %}}", self.partial).into())?;
            let array = range.evaluate()?;

            if !array.is_empty() {
                let len = array.len();
                for (i, v) in array.into_iter().enumerate() {
                    let forloop = ForloopObject::new(i, len);

                    let mut root = std::collections::HashMap::new();
                    for (id, val) in &self.vars {
                        let value = val
                            .try_evaluate(runtime)
                            .ok_or_else(|| Error::with_msg("failed to evaluate value"))?;

                        root.insert(id.as_ref(), value);
                    }
                    root.insert("forloop".into(), liquid_core::ValueCow::Borrowed(&forloop));
                    root.insert(var_name.as_ref(), v);

                    let scope = GlobalFrame::new(SandboxedStackFrame::new(runtime, &root));

                    let partial = scope
                        .partials()
                        .get(&name)
                        .or_else(|_| scope.partials().get(&format!("{name}.liquid")))
                        .trace_with(|| format!("{{% render {} %}}", self.partial).into())?;

                    partial
                        .render_to(writer, &scope)
                        .trace_with(|| format!("{{% render {} %}}", self.partial).into())
                        .context_key("index")
                        .value_with(|| format!("{}", i + 1).into())?;

                    // given that we're at the end of the loop body
                    // already, dealing with a `continue` signal is just
                    // clearing the interrupt and carrying on as normal. A
                    // `break` requires some special handling, though.
                    let current_interrupt =
                        scope.registers().get_mut::<InterruptRegister>().reset();
                    if let Some(Interrupt::Break) = current_interrupt {
                        break;
                    }
                }
            }
        } else {
            let mut root = std::collections::HashMap::new();
            for (id, val) in &self.vars {
                let value = val
                    .try_evaluate(runtime)
                    .ok_or_else(|| Error::with_msg("failed to evaluate value"))?;

                root.insert(id.as_ref(), value);
            }

            let scope = GlobalFrame::new(SandboxedStackFrame::new(runtime, &root));

            let partial = scope
                .partials()
                .get(&name)
                .or_else(|_| scope.partials().get(&format!("{name}.liquid")))
                .trace_with(|| format!("{{% render {} %}}", self.partial).into())?;

            partial
                .render_to(writer, &scope)
                .trace_with(|| format!("{{% render {} %}}", self.partial).into())
                .context_key_with(|| self.partial.to_string().into())
                .value_with(|| name.to_string().into())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use liquid_core::{
        parser,
        partials::{self, PartialCompiler},
        runtime::{self, RuntimeBuilder},
        Display_filter, Filter, FilterReflection, ParseFilter, Value,
    };

    use crate::stdlib::{self, AssignTag};

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

        fn try_get<'a>(&'a self, name: &str) -> Option<std::borrow::Cow<'a, str>> {
            match name {
                "example.txt" => Some(r#"{{'whooo' | size}}{%comment%}What happens{%endcomment%} {%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}"#.into()),
                "example_var.txt" => Some(r#"{{example_var}}"#.into()),
                "example_multi_var.txt" => Some(r#"{{example_var}} {{example}}"#.into()),
                "missing_extension.liquid" => Some(r#"{{example_var}}"#.into()),
                _ => None
            }
        }
    }

    fn options() -> Language {
        let mut options = Language::default();
        options.tags.register("render".to_owned(), RenderTag.into());
        options.tags.register("assign".to_owned(), AssignTag.into());
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
    fn render_for() {
        let text = "{% render 'example_var.txt' for (0..5) as example_var %}";
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
        assert_eq!(output, "012345");
    }

    #[test]
    fn render_with() {
        let text = "{% render 'example_var.txt' with \"hello\" as example_var %}";
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
    fn render_scope() {
        let text = "{% assign numTwo = 10 %}{% render 'example.txt', num: 5 %}";
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
        let output = template.render(&runtime);
        assert!(output.is_err());
    }

    #[test]
    fn render_tag_quotes() {
        let text = "{% render 'example.txt', num: 5, numTwo: 10 %}";
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
        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "5 wat wot");
    }

    #[test]
    fn render_variable() {
        let text = "{% render 'example_var.txt', example_var:\"hello\" %}";
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
    fn include_multiple_variables() {
        let text = "{% render 'example_multi_var.txt', example_var:\"hello\", example:\"world\" %}";
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
    fn include_multiple_variables_trailing_comma() {
        let text = "{% render 'example_multi_var.txt', example_var:\"hello\", example:\"dogs\", %}";
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
        assert_eq!(output, "hello dogs");
    }

    #[test]
    fn no_file() {
        let text = "{% render 'file_does_not_exist.liquid' %}";
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

    #[test]
    fn without_extension() {
        let text = "{% render 'missing_extension', example_var:\"hello\" %}";
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
}
