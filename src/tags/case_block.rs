use Renderable;
use LiquidOptions;
use error::{Error, Result};
use context::Context;
use lexer::Element::{self, Tag};
use token::Token::{self, Or, Identifier};
use parser::{parse, consume_value_token, split_block};

use template::Template;
use value::Value;

struct CaseOption {
    tokens: Vec<Token>,
    template: Template,
}

impl CaseOption {
    fn new(tokens: Vec<Token>, template: Template) -> CaseOption {
        CaseOption {
            tokens: tokens,
            template: template,
        }
    }

    fn evaluate(&self, value: &Value, context: &Context) -> Result<bool> {
        for t in &self.tokens {
            match try!(context.evaluate(t)) {
                Some(ref v) if *v == *value => return Ok(true),
                _ => {}
            }
        }
        Ok(false)
    }
}

struct Case {
    target: Token,
    cases: Vec<CaseOption>,
    else_block: Option<Template>,
}

impl Renderable for Case {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        if let Some(value) = try!(context.evaluate(&self.target)) {
            for case in &self.cases {
                if try!(case.evaluate(&value, context)) {
                    return case.template.render(context);
                }
            }
        }

        if let Some(ref t) = self.else_block {
            return t.render(context);
        }

        Ok(None)
    }
}

enum Conditional {
    Cond(Vec<Token>),
    Else,
}

fn parse_condition(element: &Element) -> Result<Conditional> {
    if let Tag(ref tokens, _) = *element {
        match tokens[0] {
            Identifier(ref name) if name == "else" => return Ok(Conditional::Else),

            Identifier(ref name) if name == "when" => {
                let mut values: Vec<Token> = Vec::new();
                let mut args = tokens[1..].iter();

                values.push(try!(consume_value_token(&mut args)));

                loop {
                    match args.next() {
                        Some(&Or) => {}
                        Some(x) => return Error::parser("or", Some(x)),
                        None => break,
                    }

                    values.push(try!(consume_value_token(&mut args)))
                }

                return Ok(Conditional::Cond(values));
            }

            ref x => return Error::parser("else | when", Some(x)),
        }
    } else {
        Err(Error::Parser("Expected else | when".to_owned()))
    }
}

pub fn case_block(_tag_name: &str,
                  arguments: &[Token],
                  tokens: &[Element],
                  options: &LiquidOptions)
                  -> Result<Box<Renderable>> {
    let delims = &["when", "else"];
    let mut args = arguments.iter();
    let value = try!(consume_value_token(&mut args));

    // fast forward to the first arm of the case block,
    let mut children = match split_block(&tokens[..], delims, options) {
        (_, Some(split)) => split.trailing,
        _ => return Err(Error::Parser("Expected case | else".to_owned())),
    };

    let mut result = Case {
        target: value,
        cases: Vec::new(),
        else_block: None,
    };

    loop {
        let (leading, trailing) = split_block(&children[1..], delims, options);
        let template = Template::new(try!(parse(leading, options)));

        match try!(parse_condition(&children[0])) {
            Conditional::Cond(conds) => {
                result.cases.push(CaseOption::new(conds, template));
            }
            Conditional::Else => {
                if result.else_block.is_none() {
                    result.else_block = Some(template)
                } else {
                    return Err(Error::Parser("Only one else block allowed".to_owned()));
                }
            }
        }

        match trailing {
            Some(split) => children = split.trailing,
            None => break,
        }
    }

    Ok(Box::new(result))
}

#[cfg(test)]
mod test {
    use LiquidOptions;
    use Renderable;
    use parse;
    use value::Value;
    use context::Context;

    #[test]
    fn test_case_block() {
        let text = concat!("{% case x %}",
                           "{% when 2 %}",
                           "two",
                           "{% when 3 or 4 %}",
                           "three and a half",
                           "{% else %}",
                           "otherwise",
                           "{% endcase %}");

        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();
        context.set_val("x", Value::Num(2f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("two".to_owned()));

        context.set_val("x", Value::Num(3f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("three and a half".to_owned()));

        context.set_val("x", Value::Num(4f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("three and a half".to_owned()));


        context.set_val("x", Value::str("nope"));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("otherwise".to_owned()));
    }

    #[test]
    fn test_no_matches_returns_empty_string() {
        let text = concat!("{% case x %}",
                           "{% when 2 %}",
                           "two",
                           "{% when 3 or 4 %}",
                           "three and a half",
                           "{% endcase %}");

        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();
        context.set_val("x", Value::str("nope"));
        assert_eq!(template.render(&mut context).unwrap(), Some("".to_owned()));
    }

    #[test]
    fn multiple_else_blocks_is_an_error() {
        let text = concat!("{% case x %}",
                           "{% when 2 %}",
                           "two",
                           "{% else %}",
                           "else #1",
                           "{% else %}",
                           "else # 2",
                           "{% endcase %}");

        assert!(parse(text, LiquidOptions::default()).is_err());
    }
}
