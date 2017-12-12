use Context;
use LiquidOptions;
use error::{Error, Result};

use syntax::Output;
use syntax::Renderable;
use syntax::Token;
use syntax::{parse_output, expect};

struct Assign {
    dst: String,
    src: Output,
}

impl Renderable for Assign {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = try!(self.src.apply_filters(context));
        context.set_val(&self.dst, value);
        Ok(None)
    }
}

pub fn assign_tag(_tag_name: &str,
                  arguments: &[Token],
                  _options: &LiquidOptions)
                  -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let dst = match args.next() {
        Some(&Token::Identifier(ref id)) => id.clone(),
        x => return Error::parser("Identifier", x),
    };

    expect(&mut args, &Token::Assignment)?;

    let src = parse_output(&arguments[2..])?;

    Ok(Box::new(Assign { dst: dst, src: src }))
}

#[cfg(test)]
mod test {
    use super::*;
    use syntax::Value;
    use super::super::super::parse;

    #[test]
    fn assignment_in_loop_persists_on_loop_exit() {
        let text = concat!("{% assign freestyle = false %}",
                           "{% for t in tags %}{% if t == 'freestyle' %}",
                           "{% assign freestyle = true %}",
                           "{% endif %}{% endfor %}",
                           "{% if freestyle %}",
                           "<p>Freestyle!</p>",
                           "{% endif %}");
        let template = parse(text, Default::default()).unwrap();

        // test one: no matching value in `tags`
        {
            let mut context = Context::new();
            context.set_val("tags",
                            Value::Array(vec![Value::str("alpha"),
                                              Value::str("beta"),
                                              Value::str("gamma")]));

            let output = template.render(&mut context);
            assert_eq!(context.get_val("freestyle"), Some(&Value::Bool(false)));
            assert_eq!(output.unwrap(), Some("".to_string()));
        }

        // test two: matching value in `tags`
        {
            let mut context = Context::new();
            context.set_val("tags",
                            Value::Array(vec![Value::str("alpha"),
                                              Value::str("beta"),
                                              Value::str("freestyle"),
                                              Value::str("gamma")]));

            let output = template.render(&mut context);
            assert_eq!(context.get_val("freestyle"), Some(&Value::Bool(true)));
            assert_eq!(output.unwrap(), Some("<p>Freestyle!</p>".to_string()));
        }
    }
}
