use error::{Error, Result};

use syntax::Context;
use syntax::LiquidOptions;
use syntax::Element;
use syntax::Renderable;
use syntax::Template;
use syntax::Token;
use syntax::parse;
use value::Value;

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

        context.set_val(&self.id, Value::Str(output));
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
    use syntax;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert("capture".to_owned(),
                              (capture_block as syntax::FnParseBlock).into());
        options
    }

    #[test]
    fn test_capture() {
        let text = concat!("{% capture attribute_name %}",
                           "{{ item }}-{{ i }}-color",
                           "{% endcapture %}");
        let tokens = syntax::tokenize(text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options)
            .map(syntax::Template::new)
            .unwrap();

        let mut ctx = Context::new();
        ctx.set_val("item", Value::str("potato"));
        ctx.set_val("i", Value::Num(42f32));

        let output = template.render(&mut ctx).unwrap();
        assert_eq!(ctx.get_val("attribute_name"),
                   Some(&Value::str("potato-42-color")));
        assert_eq!(output, Some("".to_owned()));
    }

    #[test]
    fn trailing_tokens_are_an_error() {
        let text = concat!("{% capture foo bar baz %}",
                           "We should never see this",
                           "{% endcapture %}");
        let tokens = syntax::tokenize(text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options).map(syntax::Template::new);
        assert!(template.is_err());
    }
}
