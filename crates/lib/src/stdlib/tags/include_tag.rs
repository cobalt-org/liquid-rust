use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::{KString, KStringRef};
use liquid_core::Expression;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::ValueCow;
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
        let partial = arguments.expect_next("Identifier or literal expected.")?;

        let partial = partial.expect_value().into_result()?;

        let mut token = arguments.next();
        let mut variable_name = None;
        let mut alias = None;
        let mut is_for_loop = false;
        let mut vars: Vec<(KString, Expression)> = Vec::new();

        match token.as_ref().map(|t| t.as_str()) {
            Some("with") => {
                variable_name = Some(
                    arguments
                        .expect_next("expected value")?
                        .expect_value()
                        .into_result()?,
                );
                token = arguments.next();
            }
            Some("for") => {
                is_for_loop = true;
                variable_name = Some(
                    arguments
                        .expect_next("expected value")?
                        .expect_value()
                        .into_result()?,
                );
                token = arguments.next();
            }
            _ => {}
        }

        if token.as_ref().map(|t| t.as_str()) == Some("as") {
            alias = Some(
                arguments
                    .expect_next("expected identifier")?
                    .expect_identifier()
                    .into_result()?
                    .to_owned()
                    .into(),
            );
            token = arguments.next();
        }

        if token.as_ref().map(|t| t.as_str()) == Some(",") {
            token = arguments.next();
        }

        while let Some(next) = token {
            let id = next.expect_identifier().into_result()?.to_owned();

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
            if token.as_ref().map(|t| t.as_str()) == Some(",") {
                token = arguments.next();
            }
        }

        Ok(Box::new(Include {
            partial,
            variable_name,
            alias,
            is_for_loop,
            vars,
        }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Include {
    partial: Expression,
    variable_name: Option<Expression>,
    alias: Option<KString>,
    is_for_loop: bool,
    vars: Vec<(KString, Expression)>,
}

impl Renderable for Include {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        if runtime.registers().in_render_tag_scope() {
            return Error::with_msg("include usage is not allowed in this context").into_err();
        }

        let value = self.partial.evaluate(runtime)?;
        let Some(scalar) = value.as_scalar() else {
            return Error::with_msg("Argument error in tag 'include' - Illegal template name")
                .into_err();
        };
        if !scalar.is_string() {
            return Error::with_msg("Argument error in tag 'include' - Illegal template name")
                .into_err();
        }
        let name = scalar.into_cow_str().into_owned();

        let default_alias = std::path::Path::new(&name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| name.rsplit('/').next().unwrap_or(&name))
            .to_owned();
        let context_variable_name: KStringRef<'_> = self
            .alias
            .as_ref()
            .map(|name| name.as_ref())
            .unwrap_or(default_alias.as_str().into());

        let variable = if let Some(variable_name) = &self.variable_name {
            Some(variable_name.evaluate(runtime)?)
        } else {
            None
        };

        if self.is_for_loop {
            if let Some(array) = variable.as_ref().and_then(|value| value.as_array()) {
                for item in array.values() {
                    render_include_partial(
                        writer,
                        runtime,
                        &name,
                        context_variable_name.as_ref(),
                        Some(ValueCow::Borrowed(item)),
                        &self.vars,
                        &self.partial,
                    )?;
                }
            } else {
                render_include_partial(
                    writer,
                    runtime,
                    &name,
                    context_variable_name.as_ref(),
                    variable,
                    &self.vars,
                    &self.partial,
                )?;
            }
        } else {
            render_include_partial(
                writer,
                runtime,
                &name,
                context_variable_name.as_ref(),
                variable,
                &self.vars,
                &self.partial,
            )?;
        }

        Ok(())
    }
}

fn render_include_partial<'v>(
    writer: &mut dyn Write,
    runtime: &dyn Runtime,
    name: &str,
    context_variable_name: &str,
    variable: Option<ValueCow<'v>>,
    vars: &[(KString, Expression)],
    partial_expression: &Expression,
) -> Result<()> {
    let mut pass_through = std::collections::HashMap::<KString, ValueCow<'v>>::new();

    for (id, val) in vars {
        let value = val.evaluate(runtime)?;
        pass_through.insert(id.clone(), value);
    }

    if let Some(value) = variable {
        pass_through.insert(KString::from_ref(context_variable_name), value);
    }

    let mut live_scope = liquid_core::runtime::LiveScopeFrame::new();
    for (name, value) in &pass_through {
        live_scope.insert(name.clone(), value.as_view());
    }
    let _live_scope_guard = liquid_core::runtime::push_live_scope_frame(runtime, live_scope);

    let scope = StackFrame::new(runtime, &pass_through);
    let partial = match scope
        .partials()
        .get(name)
        .trace_with(|| format!("{{% include {} %}}", partial_expression).into())?
    {
        Some(partial) => partial,
        None => match scope
            .partials()
            .get(&format!("{name}.liquid"))
            .trace_with(|| format!("{{% include {} %}}", partial_expression).into())?
        {
            Some(partial) => partial,
            None => return Err(missing_partial_error(scope.partials(), name)),
        },
    };

    let _scope_guard = liquid_core::runtime::enter_render_scope(&scope)?;
    liquid_core::runtime::reset_resource_limits(&scope)?;
    partial
        .render_to(writer, &scope)
        .trace_with(|| format!("{{% include {} %}}", partial_expression).into())
        .context_key_with(|| partial_expression.to_string().into())
        .value_with(|| name.to_string().into())?;

    Ok(())
}

fn missing_partial_error(partials: &dyn liquid_core::runtime::PartialStore, name: &str) -> Error {
    let available = partials.names();
    let mut available = available;
    available.sort_unstable();
    let available = itertools::join(available, ", ");
    Error::with_msg("Unknown partial-template")
        .context("requested partial", name.to_owned())
        .context("available partials", available)
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
        fn names(&self) -> Vec<&str> {
            vec![]
        }

        fn get<'a>(&'a self, name: &str) -> Result<Option<borrow::Cow<'a, str>>> {
            Ok(match name {
                "example.txt" => Some(r#"{{'whooo' | size}}{%comment%}What happens{%endcomment%} {%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}"#.into()),
                "example_var.txt" => Some(r#"{{example_var}}"#.into()),
                "example_multi_var.txt" => Some(r#"{{example_var}} {{example}}"#.into()),
                "product.liquid" => Some(r#"{{product}}"#.into()),
                "missing_extension.liquid" => Some(r#"{{example_var}}"#.into()),
                _ => None,
            })
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
    fn include_tag_quotes() {
        let text = "{% include 'example.txt' %}";
        let mut options = options();
        std::sync::Arc::make_mut(&mut options.filters)
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
        let text = "{% include 'example_var.txt' example_var:\"hello\" %}";
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
        let text = "{% include 'example_multi_var.txt' example_var:\"hello\", example:\"world\" %}";
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
        let text = "{% include 'example_multi_var.txt' example_var:\"hello\", example:\"dogs\", %}";
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
    fn include_falls_back_to_liquid_extension() {
        let text = "{% include 'missing_extension' example_var:\"hello\" %}";
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
    fn include_uses_template_stem_as_default_alias() {
        let text = "{% include 'product.liquid' with product %}";
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
        runtime.set_global("product".into(), Value::scalar("Draft 151cm"));
        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "Draft 151cm");
    }

    #[test]
    fn no_file() {
        let text = "{% include 'file_does_not_exist.liquid' %}";
        let mut options = options();
        std::sync::Arc::make_mut(&mut options.filters)
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
