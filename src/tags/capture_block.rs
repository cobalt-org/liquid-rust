use context::Context;
use error::{Error, Result};
use lexer::Element;
use LiquidOptions;
use Renderable;
use template::Template;
use token::Token::{self, Identifier};
use value::Value;
use parser::parse;

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
                     tokens: Vec<Element>,
                     options: &LiquidOptions)
                     -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let id = match args.next() {
        Some(&Identifier(ref x)) => x.clone(),
        x @ Some(_) | x @ None => return Error::parser("Identifier", x),
    };

    // there should be no trailing tokens after this
    if let t @ Some(_) = args.next() {
        return Error::parser("%}", t);
    };

    let t = Template::new(try!(parse(&tokens, options)));

    Ok(Box::new(Capture {
        id: id,
        template: t,
    }))
}

#[cfg(test)]
mod test {
    use parse;
    use LiquidOptions;
    use Renderable;
    use value::Value;
    use std::default::Default;
    use context::Context;

    #[test]
    fn test_capture() {
        let text = concat!("{% capture attribute_name %}",
                           "{{ item | upcase }}-{{ i }}-color",
                           "{% endcapture %}");
        let template = parse(text, LiquidOptions::default()).unwrap();

        let mut ctx = Context::new();
        ctx.set_val("item", Value::str("potato"));
        ctx.set_val("i", Value::Num(42f32));

        let output = template.render(&mut ctx);
        assert_eq!(output.unwrap(), Some("".to_owned()));
        assert_eq!(ctx.get_val("attribute_name"),
                   Some(&Value::str("POTATO-42-color")));
    }

    #[test]
    fn trailing_tokens_are_an_error() {
        let text = concat!("{% capture foo bar baz %}",
                           "We should never see this",
                           "{% endcapture %}");
        assert!(parse(text, LiquidOptions::default()).is_err());
    }
}
