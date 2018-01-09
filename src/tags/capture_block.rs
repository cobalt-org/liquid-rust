use error::{Error, Result};

use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;
use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use compiler::parse;
use value::Value;

#[derive(Debug)]
struct Capture {
    id: String,
    template: Template,
}

impl Renderable for Capture {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let output = match self.template.render(context) {
            Ok(Some(s)) => s.clone(),
            Ok(None) => "".to_owned(),
            Err(x) => return Err(x),
        };

        context.set_global_val(&self.id, Value::scalar(output));
        Ok(None)
    }
}

pub fn capture_block(_tag_name: &str,
                     arguments: &[Token],
                     tokens: &[Element],
                     options: &LiquidOptions)
                     -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let id = match args.next() {
        Some(&Token::Identifier(ref x)) => x.clone(),
        x @ Some(_) | x @ None => return Error::parser("Identifier", x),
    };

    // there should be no trailing tokens after this
    if let t @ Some(_) = args.next() {
        return Error::parser("%}", t);
    };

    let t = Template::new(parse(tokens, options)?);

    Ok(Box::new(Capture {
                    id: id,
                    template: t,
                }))
}

#[cfg(test)]
mod test {
    use super::*;
    use interpreter;
    use compiler;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert("capture".to_owned(),
                              (capture_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn test_capture() {
        let text = concat!("{% capture attribute_name %}",
                           "{{ item }}-{{ i }}-color",
                           "{% endcapture %}");
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        ctx.set_global_val("item", Value::scalar("potato"));
        ctx.set_global_val("i", Value::scalar(42f32));

        let output = template.render(&mut ctx).unwrap();
        assert_eq!(ctx.get_val("attribute_name"),
                   Some(&Value::scalar("potato-42-color")));
        assert_eq!(output, Some("".to_owned()));
    }

    #[test]
    fn trailing_tokens_are_an_error() {
        let text = concat!("{% capture foo bar baz %}",
                           "We should never see this",
                           "{% endcapture %}");
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options).map(interpreter::Template::new);
        assert!(template.is_err());
    }
}
