use error::{Error, Result};

use interpreter::Argument;
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;
use syntax::Element;
use syntax::LiquidOptions;
use syntax::Token;
use syntax::{parse, consume_value_token, split_block};
use value::Value;

#[derive(Debug)]
struct CaseOption {
    args: Vec<Argument>,
    template: Template,
}

impl CaseOption {
    fn new(args: Vec<Argument>, template: Template) -> CaseOption {
        CaseOption { args, template }
    }

    fn evaluate(&self, value: &Value, context: &Context) -> Result<bool> {
        for a in &self.args {
            let v = a.evaluate(&context)?;
            if v == *value {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Debug)]
struct Case {
    target: Argument,
    cases: Vec<CaseOption>,
    else_block: Option<Template>,
}

impl Renderable for Case {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let value = self.target.evaluate(context)?;
        for case in &self.cases {
            if case.evaluate(&value, context)? {
                return case.template.render(context);
            }
        }

        if let Some(ref t) = self.else_block {
            return t.render(context);
        }

        Ok(None)
    }
}

enum Conditional {
    Cond(Vec<Argument>),
    Else,
}

fn parse_condition(element: &Element) -> Result<Conditional> {
    if let Element::Tag(ref tokens, _) = *element {
        match tokens[0] {
            Token::Identifier(ref name) if name == "else" => return Ok(Conditional::Else),

            Token::Identifier(ref name) if name == "when" => {
                let mut values: Vec<Argument> = Vec::new();
                let mut args = tokens[1..].iter();

                values.push(consume_value_token(&mut args)?.to_arg()?);

                loop {
                    match args.next() {
                        Some(&Token::Or) => {}
                        Some(x) => return Error::parser("or", Some(x)),
                        None => break,
                    }

                    values.push(consume_value_token(&mut args)?.to_arg()?);
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
    let value = consume_value_token(&mut args)?.to_arg()?;

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
        let template = Template::new(parse(leading, options)?);

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
    use super::*;
    use interpreter;
    use syntax;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert("case".to_owned(),
                              (case_block as syntax::FnParseBlock).into());
        options
    }

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
        let tokens = syntax::tokenize(text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

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
        let tokens = syntax::tokenize(text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

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
        let tokens = syntax::tokenize(text).unwrap();
        let options = options();
        let template = syntax::parse(&tokens, &options).map(interpreter::Template::new);
        assert!(template.is_err());
    }
}
