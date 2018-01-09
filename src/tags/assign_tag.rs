use error::{Error, Result};

use interpreter::Context;
use interpreter::Output;
use interpreter::Renderable;
use compiler::LiquidOptions;
use compiler::Token;
use compiler::{parse_output, expect};

#[derive(Clone, Debug)]
struct Assign {
    dst: String,
    src: Output,
}

impl Renderable for Assign {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = self.src.apply_filters(context)?;
        context.set_global_val(&self.dst, value);
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
    use compiler;
    use interpreter;
    use value::Value;
    use tags;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.tags.insert("assign".to_owned(),
                            (assign_tag as compiler::FnParseTag).into());
        options.blocks.insert("if".to_owned(),
                              (tags::if_block as compiler::FnParseBlock).into());
        options.blocks.insert("for".to_owned(),
                              (tags::for_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn assignment_in_loop_persists_on_loop_exit() {
        let text = concat!("{% assign freestyle = false %}",
                           "{% for t in tags %}{% if t == 'freestyle' %}",
                           "{% assign freestyle = true %}",
                           "{% endif %}{% endfor %}",
                           "{% if freestyle %}",
                           "<p>Freestyle!</p>",
                           "{% endif %}");
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        // test one: no matching value in `tags`
        {
            let mut context = Context::new();
            context.set_global_val("tags",
                                   Value::Array(vec![Value::scalar("alpha"),
                                                     Value::scalar("beta"),
                                                     Value::scalar("gamma")]));

            let output = template.render(&mut context).unwrap();
            assert_eq!(context.get_val("freestyle"), Some(&Value::scalar(false)));
            assert_eq!(output, Some("".to_string()));
        }

        // test two: matching value in `tags`
        {
            let mut context = Context::new();
            context.set_global_val("tags",
                                   Value::Array(vec![Value::scalar("alpha"),
                                                     Value::scalar("beta"),
                                                     Value::scalar("freestyle"),
                                                     Value::scalar("gamma")]));

            let output = template.render(&mut context).unwrap();
            assert_eq!(context.get_val("freestyle"), Some(&Value::scalar(true)));
            assert_eq!(output, Some("<p>Freestyle!</p>".to_string()));
        }
    }
}
