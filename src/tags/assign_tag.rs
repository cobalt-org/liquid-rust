use std::io::Write;

use liquid_error::Result;
use liquid_error::ResultLiquidExt;

use compiler::FilterChain;
use compiler::Language;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;

#[derive(Debug)]
struct Assign {
    dst: String,
    src: FilterChain,
}

impl Assign {
    fn trace(&self) -> String {
        format!("{{% assign {} = {}%}}", self.dst, self.src)
    }
}

impl Renderable for Assign {
    fn render_to(&self, _writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = self
            .src
            .evaluate(context)
            .trace_with(|| self.trace().into())?;
        context.stack_mut().set_global(self.dst.to_owned(), value);
        Ok(())
    }
}

pub fn assign_tag(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    options: &Language,
) -> Result<Box<Renderable>> {
    let dst = arguments
        .expect_next("Identifier expected.")?
        .expect_identifier()
        .into_result()?
        .to_string();

    arguments
        .expect_next("Assignment operator \"=\" expected.")?
        .expect_str("=")
        .into_result_custom_msg("Assignment operator \"=\" expected.")?;

    let src = arguments
        .expect_next("FilterChain expected.")?
        .expect_filter_chain(options)
        .into_result()?;

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    Ok(Box::new(Assign { dst, src }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;
    use value::Scalar;
    use value::Value;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("assign", (assign_tag as compiler::FnParseTag).into());
        options
            .blocks
            .register("if", (tags::if_block as compiler::FnParseBlock).into());
        options
            .blocks
            .register("for", (tags::for_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn assign() {
        let options = options();
        let template = compiler::parse("{% assign freestyle = false %}{{ freestyle }}", &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();

        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "false");
    }

    #[test]
    fn assign_array_indexing() {
        let text = concat!("{% assign freestyle = tags[1] %}", "{{ freestyle }}");
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global(
            "tags",
            Value::Array(vec![
                Value::scalar("alpha"),
                Value::scalar("beta"),
                Value::scalar("gamma"),
            ]),
        );

        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "beta");
    }

    #[test]
    fn assign_object_indexing() {
        let text = concat!(
            r#"{% assign freestyle = tags["greek"] %}"#,
            "{{ freestyle }}"
        );
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global(
            "tags",
            Value::Object(
                vec![("greek".into(), Value::scalar("alpha"))]
                    .into_iter()
                    .collect(),
            ),
        );

        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "alpha");
    }

    #[test]
    fn assign_in_loop_persists_on_loop_exit() {
        let text = concat!(
            "{% assign freestyle = false %}",
            "{% for t in tags %}{% if t == 'freestyle' %}",
            "{% assign freestyle = true %}",
            "{% endif %}{% endfor %}",
            "{% if freestyle %}",
            "<p>Freestyle!</p>",
            "{% endif %}"
        );

        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        // test one: no matching value in `tags`
        {
            let mut context = Context::new();
            context.stack_mut().set_global(
                "tags",
                Value::Array(vec![
                    Value::scalar("alpha"),
                    Value::scalar("beta"),
                    Value::scalar("gamma"),
                ]),
            );

            let output = template.render(&mut context).unwrap();
            assert_eq!(
                context.stack().get(&[Scalar::new("freestyle")]).unwrap(),
                &Value::scalar(false)
            );
            assert_eq!(output, "");
        }

        // test two: matching value in `tags`
        {
            let mut context = Context::new();
            context.stack_mut().set_global(
                "tags",
                Value::Array(vec![
                    Value::scalar("alpha"),
                    Value::scalar("beta"),
                    Value::scalar("freestyle"),
                    Value::scalar("gamma"),
                ]),
            );

            let output = template.render(&mut context).unwrap();
            assert_eq!(
                context.stack().get(&[Scalar::new("freestyle")]).unwrap(),
                &Value::scalar(true)
            );
            assert_eq!(output, "<p>Freestyle!</p>");
        }
    }
}
