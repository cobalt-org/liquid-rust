use itertools;

use error::{Error, Result};

use interpreter::Argument;
use interpreter::Context;
use interpreter::Renderable;
use syntax::Token;
use syntax::LiquidOptions;
use syntax::{consume_value_token, value_token};

#[derive(Clone, Debug)]
struct Cycle {
    name: String,
    values: Vec<Argument>,
}

impl Renderable for Cycle {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = context.cycle_element(&self.name, &self.values)?;
        Ok(value.map(|v| v.to_string()))
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
        Some(&Token::Comma) |
        None => {
            // first argument is the first item in the cycle
            values.push(first.to_arg()?);
        }
        x => return Error::parser(": | Number | String | Identifier", x),
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
            x => return Error::parser("Comma", x),
        }
    }

    if name.is_empty() {
        name = itertools::join(values.iter(), "-");
    }
    println!("name={}", name);

    Ok(Cycle {
           name: name,
           values: values,
       })
}

pub fn cycle_tag(_tag_name: &str,
                 arguments: &[Token],
                 options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    parse_cycle(arguments, options).map(|opt| Box::new(opt) as Box<Renderable>)
}

#[cfg(test)]
mod test {
    use super::*;
    use value::Value;
    use syntax;
    use interpreter;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .tags
            .insert("cycle".to_owned(), (cycle_tag as syntax::FnParseTag).into());
        options
    }

    #[test]
    fn unnamed_cycle_gets_a_name() {
        let tokens = vec![Token::Identifier("this".to_owned()),
                          Token::Comma,
                          Token::StringLiteral("cycle".to_owned()),
                          Token::Comma,
                          Token::Identifier("has".to_owned()),
                          Token::Comma,
                          Token::Identifier("no".to_owned()),
                          Token::Comma,
                          Token::Identifier("name".to_owned())];
        let options = LiquidOptions::default();
        let cycle = parse_cycle(&tokens[..], &options).unwrap();
        assert!(!cycle.name.is_empty());
    }

    #[test]
    fn named_values_are_independent() {
        let text = concat!("{% cycle 'a': 'one', 'two', 'three' %}\n",
                           "{% cycle 'a': 'one', 'two', 'three' %}\n",
                           "{% cycle 'b': 'one', 'two', 'three' %}\n",
                           "{% cycle 'b': 'one', 'two', 'three' %}\n")
            .to_owned();
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), Some("one\ntwo\none\ntwo\n".to_owned()));
    }

    #[test]
    fn values_are_cycled() {
        let text = concat!("{% cycle 'one', 'two', 'three' %}\n",
                           "{% cycle 'one', 'two', 'three' %}\n",
                           "{% cycle 'one', 'two', 'three' %}\n",
                           "{% cycle 'one', 'two', 'three' %}\n")
            .to_owned();
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), Some("one\ntwo\nthree\none\n".to_owned()));
    }

    #[test]
    fn values_can_be_variables() {
        let text = concat!("{% cycle alpha, beta, gamma %}\n",
                           "{% cycle alpha, beta, gamma %}\n",
                           "{% cycle alpha, beta, gamma %}\n",
                           "{% cycle alpha, beta, gamma %}\n")
            .to_owned();
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_val("alpha", Value::Num(1f32));
        context.set_val("beta", Value::Num(2f32));
        context.set_val("gamma", Value::Num(3f32));

        let output = template.render(&mut context);

        assert_eq!(output.unwrap(), Some("1\n2\n3\n1\n".to_owned()));
    }

    #[test]
    fn bad_cycle_indices_dont_crash() {
        // note the pair of cycle tags with the same name but a differing
        // number of elements
        let text = concat!("{% cycle c: 1, 2 %}\n", "{% cycle c: 1 %}\n").to_owned();

        let tokens = syntax::tokenize(&text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();
        let output = template.render(&mut Default::default());
        assert!(output.is_err());
    }
}
