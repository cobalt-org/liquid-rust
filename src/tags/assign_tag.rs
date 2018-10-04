use std::io::Write;

use liquid_error::Result;
use liquid_error::ResultLiquidExt;

use compiler::LiquidOptions;
use compiler::Token;
use compiler::{expect, parse_output, unexpected_token_error};
use interpreter::Context;
use interpreter::FilterChain;
use interpreter::Renderable;

#[derive(Clone, Debug)]
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
        let value = self.src.evaluate(context).trace_with(|| self.trace())?;
        context
            .stack_mut()
            .set_global(self.dst.to_owned(), value);
        Ok(())
    }
}

pub fn assign_tag(
    _tag_name: &str,
    arguments: &[Token],
    _options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let dst = match args.next() {
        Some(&Token::Identifier(ref id)) => id.clone(),
        x => return Err(unexpected_token_error("identifier", x)),
    };

    expect(&mut args, &Token::Assignment)?;

    let src = parse_output(&arguments[2..])?;

    Ok(Box::new(Assign { dst, src }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;
    use value::Value;
    use value::Index;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .tags
            .insert("assign", (assign_tag as compiler::FnParseTag).into());
        options
            .blocks
            .insert("if", (tags::if_block as compiler::FnParseBlock).into());
        options
            .blocks
            .insert("for", (tags::for_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn assignment_in_loop_persists_on_loop_exit() {
        let text = concat!(
            "{% assign freestyle = false %}",
            "{% for t in tags %}{% if t == 'freestyle' %}",
            "{% assign freestyle = true %}",
            "{% endif %}{% endfor %}",
            "{% if freestyle %}",
            "<p>Freestyle!</p>",
            "{% endif %}"
        );
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
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
                context.stack().get(&vec![Index::with_key("freestyle")].into_iter().collect()).unwrap(),
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
                context.stack().get(&vec![Index::with_key("freestyle")].into_iter().collect()).unwrap(),
                &Value::scalar(true)
            );
            assert_eq!(output, "<p>Freestyle!</p>");
        }
    }
}
