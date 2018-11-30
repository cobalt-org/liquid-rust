use std::io::Write;

use liquid_error::{Result, ResultLiquidExt};
use liquid_value::Value;

use compiler::LiquidOptions;
use compiler::TagBlock;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;

#[derive(Debug)]
struct Capture {
    id: String,
    template: Template,
}

impl Capture {
    fn trace(&self) -> String {
        format!("{{% capture {} %}}", self.id)
    }
}

impl Renderable for Capture {
    fn render_to(&self, _writer: &mut Write, context: &mut Context) -> Result<()> {
        let mut captured = Vec::new();
        self.template
            .render_to(&mut captured, context)
            .trace_with(|| self.trace())?;

        let output = String::from_utf8(captured).expect("render only writes UTF-8");
        context
            .stack_mut()
            .set_global(self.id.to_owned(), Value::scalar(output));
        Ok(())
    }
}

pub fn capture_block(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let id = arguments
        .expect_next("Identifier expected")?
        .expect_identifier()
        .into_result()?
        .to_string();

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let template = Template::new(
        tokens
            .parse_all(options)
            .trace_with(|| format!("{{% capture {} %}}", &id))?,
    );

    tokens.assert_empty();
    Ok(Box::new(Capture { id, template }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use value::Scalar;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .blocks
            .insert("capture", (capture_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn test_capture() {
        let text = concat!(
            "{% capture attribute_name %}",
            "{{ item }}-{{ i }}-color",
            "{% endcapture %}"
        );
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        ctx.stack_mut().set_global("item", Value::scalar("potato"));
        ctx.stack_mut().set_global("i", Value::scalar(42f64));

        let output = template.render(&mut ctx).unwrap();
        assert_eq!(
            ctx.stack().get(&[Scalar::new("attribute_name")]).unwrap(),
            &Value::scalar("potato-42-color")
        );
        assert_eq!(output, "");
    }

    #[test]
    fn trailing_tokens_are_an_error() {
        let text = concat!(
            "{% capture foo bar baz %}",
            "We should never see this",
            "{% endcapture %}"
        );
        let options = options();
        let template = compiler::parse(text, &options).map(interpreter::Template::new);
        assert!(template.is_err());
    }
}
