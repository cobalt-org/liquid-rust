use Renderable;
use context::Context;
use template::Template;
use LiquidOptions;
use token::Token::{self, Comparison};
use token::ComparisonOperator::{self, Equals, NotEquals, LessThan, GreaterThan, LessThanEquals,
                                GreaterThanEquals, Contains};
use parser::{parse, split_block, consume_value_token};
use lexer::Element;
use error::{Error, Result};

struct Condition {
    lh: Token,
    comparison: ComparisonOperator,
    rh: Token,
}

struct Conditional {
    condition: Condition,
    mode: bool,
    if_true: Template,
    if_false: Option<Template>,
}

impl Conditional {
    fn compare(&self, context: &Context) -> Result<bool> {
        let a = try!(context.evaluate(&self.condition.lh));
        let b = try!(context.evaluate(&self.condition.rh));

        if a == None || b == None {
            return Ok(false);
        }

        let result = match self.condition.comparison {
            Equals => a == b,
            NotEquals => a != b,
            LessThan => a < b,
            GreaterThan => a > b,
            LessThanEquals => a <= b,
            GreaterThanEquals => a >= b,
            Contains => false, // TODO!!!
        };

        Ok(result == self.mode)
    }
}

impl Renderable for Conditional {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        if try!(self.compare(context)) {
            self.if_true.render(context)
        } else {
            match self.if_false {
                Some(ref template) => template.render(context),
                _ => Ok(None),
            }
        }
    }
}

/// Common parsing for "if" and "unless" condition
fn condition(arguments: &[Token]) -> Result<Condition> {
    let mut args = arguments.iter();

    let lh = try!(consume_value_token(&mut args));

    let (comp, rh) = match args.next() {
        Some(&Comparison(ref x)) => {
            let rhs = try!(consume_value_token(&mut args));
            (x.clone(), rhs)
        }
        None => {
            // no trailing operator or RHS value implies "== true"
            (ComparisonOperator::Equals, Token::BooleanLiteral(true))
        }
        x @ Some(_) => return Error::parser("comparison operator", x),
    };

    Ok(Condition {
           lh: lh,
           comparison: comp,
           rh: rh,
       })
}

pub fn unless_block(_tag_name: &str,
                    arguments: &[Token],
                    tokens: &[Element],
                    options: &LiquidOptions)
                    -> Result<Box<Renderable>> {
    let cond = try!(condition(arguments));
    Ok(Box::new(Conditional {
                    condition: cond,
                    mode: false,
                    if_true: Template::new(try!(parse(&tokens[..], options))),
                    if_false: None,
                }))
}

pub fn if_block(_tag_name: &str,
                arguments: &[Token],
                tokens: &[Element],
                options: &LiquidOptions)
                -> Result<Box<Renderable>> {
    let cond = try!(condition(arguments));

    let (leading_tokens, trailing_tokens) = split_block(&tokens[..], &["else", "elsif"], options);
    let if_false = match trailing_tokens {
        None => None,

        Some(ref split) if split.delimiter == "else" => {
            let parsed = try!(parse(&split.trailing[1..], options));
            Some(Template::new(parsed))
        }

        Some(ref split) if split.delimiter == "elsif" => {
            let child_tokens: Vec<Element> = split.trailing.iter().skip(1).cloned().collect();
            let parsed = try!(if_block("if", &split.args[1..], &child_tokens, options));
            Some(Template::new(vec![parsed]))
        }

        Some(split) => panic!("Unexpected delimiter: {:?}", split.delimiter),
    };

    let if_true = Template::new(try!(parse(leading_tokens, options)));

    Ok(Box::new(Conditional {
                    condition: cond,
                    mode: true,
                    if_true: if_true,
                    if_false: if_false,
                }))
}

#[cfg(test)]
mod test {
    use LiquidOptions;
    use Renderable;
    use context::Context;
    use std::default::Default;
    use parse;

    #[test]
    fn number_comparison() {
        let a = parse("{% if 6 < 7  %}if true{% endif %}",
                      LiquidOptions::default())
            .unwrap()
            .render(&mut Context::new());
        assert_eq!(a.unwrap(), Some("if true".to_owned()));

        let b = parse("{% if 7 < 6  %}if true{% else %}if false{% endif %}",
                      LiquidOptions::default())
            .unwrap()
            .render(&mut Context::new());
        assert_eq!(b.unwrap(), Some("if false".to_owned()));
    }

    #[test]
    fn string_comparison() {
        // "one" == "one" then "if true" else "if false"
        let a = parse("{% if \"one\" == \"one\" %}if true{% endif %}",
                      LiquidOptions::default())
            .unwrap()
            .render(&mut Context::new());
        assert_eq!(a.unwrap(), Some("if true".to_owned()));

        // "one" == "two"
        let b = parse("{% if \"one\" == \"two\" %}if true{% endif %}",
                      LiquidOptions::default())
            .unwrap()
            .render(&mut Context::new());
        assert_eq!(b.unwrap(), Some("".to_owned()));
    }

    #[test]
    fn implicit_comparison() {
        use context::Context;
        use parse;
        use LiquidOptions;
        use Renderable;
        use value::Value;

        let text = concat!("{% if truthy %}",
                           "yep",
                           "{% else %}",
                           "nope",
                           "{% endif %}");

        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();

        // first pass, "truthy" == false
        context.set_val("truthy", Value::Bool(false));
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("nope".to_string()));

        // second pass, "truthy" == true
        context.set_val("truthy", Value::Bool(true));
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("yep".to_string()));
    }

    #[test]
    fn unless() {
        use value::Value;
        let text = concat!("{% unless some_value == 1 %}",
                           "unless body",
                           "{% endunless %}");
        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();

        context.set_val("some_value", Value::Num(1f32));
        assert_eq!(template.render(&mut context).unwrap(), Some("".to_string()));

        context.set_val("some_value", Value::Num(42f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("unless body".to_string()));
    }

    #[test]
    fn nested_if_else() {
        use value::Value;
        let text = concat!("{% if truthy %}",
                           "yep, ",
                           "{% if also_truthy %}",
                           "also truthy",
                           "{% else %}",
                           "not also truthy",
                           "{% endif %}",
                           "{% else %}",
                           "nope",
                           "{% endif %}");
        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();
        context.set_val("truthy", Value::Bool(true));
        context.set_val("also_truthy", Value::Bool(false));

        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("yep, not also truthy".to_string()));
    }

    #[test]
    fn multiple_elif_blocks() {
        use value::Value;
        let text = concat!("{% if a == 1 %}",
                           "first",
                           "{% elsif a == 2 %}",
                           "second",
                           "{% elsif a == 3 %}",
                           "third",
                           "{% else %}",
                           "fourth",
                           "{% endif %}");
        let template = parse(text, LiquidOptions::default()).unwrap();
        let mut context = Context::new();

        context.set_val("a", Value::Num(1f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("first".to_string()));

        context.set_val("a", Value::Num(2f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("second".to_string()));

        context.set_val("a", Value::Num(3f32));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("third".to_string()));

        context.set_val("a", Value::str("else"));
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("fourth".to_string()));
    }
}
