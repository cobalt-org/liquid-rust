use Renderable;
use context::Context;
use LiquidOptions;
use output::Output;
use parser::{parse_output, expect};
use token::Token::{self, Identifier, Assignment};
use error::{Error, Result};

struct Assign {
    dst: String,
    src: Output
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
                  _options: &LiquidOptions) -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let dst = match args.next() {
        Some(&Identifier(ref id)) => id.clone(),
        x => return Error::parser("Identifier", x)
    };

    try!(expect(&mut args, Assignment));

    let src = try!(parse_output(&arguments[2..]));

    Ok(Box::new(Assign {
        dst: dst,
        src: src
    }))
}

#[cfg(test)]
mod test {
    use parse;
    use Renderable;
    use context::Context;
    use value::Value;

    #[test]
    fn assignment_in_loop_persists_on_loop_exit() {
        let text = concat!(
            "{% assign freestyle = false %}",
            "{% for t in tags %}{% if t == 'freestyle' %}",
            "{% assign freestyle = true %}",
            "{% endif %}{% endfor %}",
            "{% if freestyle %}",
            "<p>Freestyle!</p>",
            "{% endif %}");
        let template = parse(text, Default::default()).unwrap();

        /* test one: no matching value in `tags` */ {
            let mut context = Context::new();
            context.set_val("tags", Value::Array(vec!(
                Value::str("alpha"),
                Value::str("beta"),
                Value::str("gamma")
            )));

            let output = template.render(&mut context);
            assert_eq!(context.get_val("freestyle"), Some(&Value::Bool(false)));
            assert_eq!(output.unwrap(), Some("".to_string()));
        }

        /* test two: matching value in `tags` */ {
            let mut context = Context::new();
            context.set_val("tags", Value::Array(vec!(
                Value::str("alpha"),
                Value::str("beta"),
                Value::str("freestyle"),
                Value::str("gamma")
            )));

            let output = template.render(&mut context);
            assert_eq!(context.get_val("freestyle"), Some(&Value::Bool(true)));
            assert_eq!(output.unwrap(), Some("<p>Freestyle!</p>".to_string()));
        }
    }
}
