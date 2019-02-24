use std::io::Write;

use liquid_error::{Result, ResultLiquidExt};

use compiler::Language;
use compiler::TagTokenIter;
use compiler::TryMatchToken;
use interpreter::Context;
use interpreter::Expression;
use interpreter::Renderable;

#[derive(Debug)]
struct Include {
    partial: Expression,
}

impl Renderable for Include {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let name = self.partial.evaluate(context)?.render().to_string();
        context.run_in_named_scope(name.clone(), |mut scope| -> Result<()> {
            let partial = scope
                .partials()
                .get(&name)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())?;
            partial
                .render_to(writer, &mut scope)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())
                .context_key_with(|| self.partial.to_string().into())
                .value_with(|| name.to_string().into())
        })?;

        Ok(())
    }
}

pub fn include_tag(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    _options: &Language,
) -> Result<Box<Renderable>> {
    let name = arguments.expect_next("Identifier or literal expected.")?;

    // This may accept strange inputs such as `{% include 0 %}` or `{% include filterchain | filter:0 %}`.
    // Those inputs would fail anyway by there being not a path with those names so they are not a big concern.
    let name = match name.expect_literal() {
        // Using `to_str()` on literals ensures `Strings` will have their quotes trimmed.
        TryMatchToken::Matches(name) => name.to_str().to_string(),
        TryMatchToken::Fails(name) => name.as_str().to_string(),
    };

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let partial = Expression::with_literal(name);

    Ok(Box::new(Include { partial }))
}

#[cfg(test)]
mod test {
    use std::borrow;

    use compiler;
    use compiler::Filter;
    use derive::*;
    use interpreter;
    use interpreter::ContextBuilder;
    use partials;
    use partials::PartialCompiler;
    use tags;
    use value;
    use value::Value;

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
                _ => None
            }
        }
    }

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("include", (include_tag as compiler::FnParseTag).into());
        options.blocks.register(
            "comment",
            (tags::comment_block as compiler::FnParseBlock).into(),
        );
        options
            .blocks
            .register("if", (tags::if_block as compiler::FnParseBlock).into());
        options
    }

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(name = "size", description = "tests helper", parsed(SizeFilter))]
    pub struct SizeFilterParser;

    #[derive(Debug, Default, Display_filter)]
    #[name = "size"]
    pub struct SizeFilter;

    impl Filter for SizeFilter {
        fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
            match *input {
                Value::Scalar(ref x) => Ok(Value::scalar(x.to_str().len() as i32)),
                Value::Array(ref x) => Ok(Value::scalar(x.len() as i32)),
                Value::Object(ref x) => Ok(Value::scalar(x.len() as i32)),
                _ => Ok(Value::scalar(0i32)),
            }
        }
    }

    #[test]
    fn include_tag_quotes() {
        let text = "{% include 'example.txt' %}";
        let mut options = options();
        options.filters.register("size", Box::new(SizeFilterParser));
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let mut context = ContextBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        context
            .stack_mut()
            .set_global("num", value::Value::scalar(5f64));
        context
            .stack_mut()
            .set_global("numTwo", value::Value::scalar(10f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "5 wat wot");
    }

    #[test]
    fn include_non_string() {
        let text = "{% include example.txt %}";
        let mut options = options();
        options.filters.register("size", Box::new(SizeFilterParser));
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let mut context = ContextBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        context
            .stack_mut()
            .set_global("num", value::Value::scalar(5f64));
        context
            .stack_mut()
            .set_global("numTwo", value::Value::scalar(10f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "5 wat wot");
    }

    #[test]
    fn no_file() {
        let text = "{% include 'file_does_not_exist.liquid' %}";
        let mut options = options();
        options.filters.register("size", Box::new(SizeFilterParser));
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let partials = partials::OnDemandCompiler::<TestSource>::empty()
            .compile(::std::sync::Arc::new(options))
            .unwrap();
        let mut context = ContextBuilder::new()
            .set_partials(partials.as_ref())
            .build();
        context
            .stack_mut()
            .set_global("num", value::Value::scalar(5f64));
        context
            .stack_mut()
            .set_global("numTwo", value::Value::scalar(10f64));
        let output = template.render(&mut context);
        assert!(output.is_err());
    }
}
