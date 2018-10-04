use std::io::Write;

use itertools;
use liquid_error::{Result, ResultLiquidChainExt, ResultLiquidExt};

use compiler::LiquidOptions;
use compiler::Token;
use compiler::{consume_value_token, unexpected_token_error, value_token};
use interpreter::Argument;
use interpreter::Context;
use interpreter::Renderable;

#[derive(Clone, Debug)]
struct Cycle {
    name: String,
    values: Vec<Argument>,
}

impl Cycle {
    fn trace(&self) -> String {
        format!(
            "{{% cycle {} %}}",
            itertools::join(self.values.iter(), ", ")
        )
    }
}

impl Renderable for Cycle {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = context
            .cycles()
            .cycle_element(&self.name, &self.values)
            .trace_with(|| self.trace())?;
        write!(writer, "{}", value).chain("Failed to render")?;
        Ok(())
    }
}

/// Internal implementation of cycle, to allow easier testing.
fn parse_cycle(arguments: &[Token], _options: &LiquidOptions) -> Result<Cycle> {
    let mut args = arguments.iter();
    let mut name = String::new();
    let mut values = Vec::new();
    let first = consume_value_token(&mut args)?;

    match args.next() {
        Some(&Token::Colon) => {
            // the first argument is the name of the cycle block
            name = first.to_string();
        }
        Some(&Token::Comma) | None => {
            // first argument is the first item in the cycle
            values.push(first.to_arg()?);
        }
        x => {
            return Err(unexpected_token_error(
                "string | number | boolean | identifier",
                x,
            ))
        }
    }

    loop {
        match args.next() {
            Some(a) => {
                let v = value_token(a.clone())?.to_arg()?;
                values.push(v);
            }
            None => break,
        }

        match args.next() {
            Some(&Token::Comma) => {}
            None => break,
            x => return Err(unexpected_token_error("`,`", x)),
        }
    }

    if name.is_empty() {
        name = itertools::join(values.iter(), "-");
    }

    Ok(Cycle { name, values })
}

pub fn cycle_tag(
    _tag_name: &str,
    arguments: &[Token],
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    parse_cycle(arguments, options).map(|opt| Box::new(opt) as Box<Renderable>)
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use value::Value;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .tags
            .insert("cycle", (cycle_tag as compiler::FnParseTag).into());
        options
    }

    #[test]
    fn unnamed_cycle_gets_a_name() {
        let tokens = vec![
            Token::Identifier("this".to_owned()),
            Token::Comma,
            Token::StringLiteral("cycle".to_owned()),
            Token::Comma,
            Token::Identifier("has".to_owned()),
            Token::Comma,
            Token::Identifier("no".to_owned()),
            Token::Comma,
            Token::Identifier("name".to_owned()),
        ];
        let options = LiquidOptions::default();
        let cycle = parse_cycle(&tokens[..], &options).unwrap();
        assert!(!cycle.name.is_empty());
    }

    #[test]
    fn named_values_are_independent() {
        let text = concat!(
            "{% cycle 'a': 'one', 'two', 'three' %}\n",
            "{% cycle 'a': 'one', 'two', 'three' %}\n",
            "{% cycle 'b': 'one', 'two', 'three' %}\n",
            "{% cycle 'b': 'one', 'two', 'three' %}\n"
        ).to_owned();
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), "one\ntwo\none\ntwo\n");
    }

    #[test]
    fn values_are_cycled() {
        let text = concat!(
            "{% cycle 'one', 'two', 'three' %}\n",
            "{% cycle 'one', 'two', 'three' %}\n",
            "{% cycle 'one', 'two', 'three' %}\n",
            "{% cycle 'one', 'two', 'three' %}\n"
        ).to_owned();
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), "one\ntwo\nthree\none\n");
    }

    #[test]
    fn values_can_be_variables() {
        let text = concat!(
            "{% cycle alpha, beta, gamma %}\n",
            "{% cycle alpha, beta, gamma %}\n",
            "{% cycle alpha, beta, gamma %}\n",
            "{% cycle alpha, beta, gamma %}\n"
        ).to_owned();
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global("alpha", Value::scalar(1f64));
        context.stack_mut().set_global("beta", Value::scalar(2f64));
        context.stack_mut().set_global("gamma", Value::scalar(3f64));

        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), "1\n2\n3\n1\n");
    }

    #[test]
    fn bad_cycle_indices_dont_crash() {
        // note the pair of cycle tags with the same name but a differing
        // number of elements
        let text = concat!("{% cycle c: 1, 2 %}\n", "{% cycle c: 1 %}\n").to_owned();

        let tokens = compiler::tokenize(&text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();
        let output = template.render(&mut Default::default());
        assert!(output.is_err());
    }
}
