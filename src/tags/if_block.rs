use error::{Error, Result};

use value::Value;
use interpreter::Argument;
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;
use compiler::ComparisonOperator;
use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use compiler::{parse, split_block, consume_value_token};

#[derive(Clone, Debug)]
struct Condition {
    lh: Argument,
    comparison: ComparisonOperator,
    rh: Argument,
}

#[derive(Debug)]
struct Conditional {
    condition: Condition,
    mode: bool,
    if_true: Template,
    if_false: Option<Template>,
}

fn contains_check(a: &Value, b: &Value) -> Result<bool> {
    let b = b.to_str();

    match *a {
        Value::Scalar(ref val) => Ok(val.to_str().contains(b.as_ref())),
        Value::Object(ref obj) => Ok(obj.contains_key(b.as_ref())),
        Value::Array(ref arr) => {
            for elem in arr {
                let elem = elem.to_str();
                if elem == b {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => {
            Error::renderer("Left-hand side of contains operator must be a string, array or object")
        }
    }
}

impl Conditional {
    fn compare(&self, context: &Context) -> Result<bool> {
        let a = self.condition.lh.evaluate(context)?;
        let b = self.condition.rh.evaluate(context)?;

        let result = match self.condition.comparison {
            ComparisonOperator::Equals => a == b,
            ComparisonOperator::NotEquals => a != b,
            ComparisonOperator::LessThan => a < b,
            ComparisonOperator::GreaterThan => a > b,
            ComparisonOperator::LessThanEquals => a <= b,
            ComparisonOperator::GreaterThanEquals => a >= b,
            ComparisonOperator::Contains => contains_check(&a, &b)?,
        };

        Ok(result == self.mode)
    }
}

impl Renderable for Conditional {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        if self.compare(context)? {
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

    let lh = consume_value_token(&mut args)?.to_arg()?;

    let (comp, rh) = match args.next() {
        Some(&Token::Comparison(ref x)) => {
            let rhs = consume_value_token(&mut args)?.to_arg()?;
            (x.clone(), rhs)
        }
        None => {
            // no trailing operator or RHS value implies "== true"
            (ComparisonOperator::Equals, Token::BooleanLiteral(true).to_arg()?)
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
    let cond = condition(arguments)?;
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
    let cond = condition(arguments)?;

    let (leading_tokens, trailing_tokens) = split_block(&tokens[..], &["else", "elsif"], options);
    let if_false = match trailing_tokens {
        None => None,

        Some(ref split) if split.delimiter == "else" => {
            let parsed = parse(&split.trailing[1..], options)?;
            Some(Template::new(parsed))
        }

        Some(ref split) if split.delimiter == "elsif" => {
            let child_tokens: Vec<Element> = split.trailing.iter().skip(1).cloned().collect();
            let parsed = if_block("if", &split.args[1..], &child_tokens, options)?;
            Some(Template::new(vec![parsed]))
        }

        Some(split) => panic!("Unexpected delimiter: {:?}", split.delimiter),
    };

    let if_true = Template::new(parse(leading_tokens, options)?);

    Ok(Box::new(Conditional {
                    condition: cond,
                    mode: true,
                    if_true: if_true,
                    if_false: if_false,
                }))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::*;
    use value::Value;
    use compiler;
    use interpreter;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .blocks
            .insert("if", (if_block as compiler::FnParseBlock).into());
        options
            .blocks
            .insert("unless", (unless_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn number_comparison() {
        let text = "{% if 6 < 7  %}if true{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));

        let text = "{% if 7 < 6  %}if true{% else %}if false{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }

    #[test]
    fn string_comparison() {
        let text = r#"{% if "one" == "one"  %}if true{% endif %}"#;
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));

        let text = r#"{% if "one" == "two"  %}if true{% else %}if false{% endif %}"#;
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }

    #[test]
    fn implicit_comparison() {
        let text = concat!("{% if truthy %}",
                           "yep",
                           "{% else %}",
                           "nope",
                           "{% endif %}");

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("truthy", Value::Nil);
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("nope".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("truthy", Value::scalar(false));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("nope".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("truthy", Value::scalar(true));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("yep".to_owned()));
    }

    #[test]
    fn unless() {
        let text = concat!("{% unless some_value == 1 %}",
                           "unless body",
                           "{% endunless %}");

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("some_value", Value::scalar(1f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("some_value", Value::scalar(42f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("unless body".to_owned()));
    }

    #[test]
    fn nested_if_else() {
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
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("truthy", Value::scalar(true));
        context.set_global_val("also_truthy", Value::scalar(false));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("yep, not also truthy".to_owned()));
    }

    #[test]
    fn multiple_elif_blocks() {
        let text = concat!("{% if a == 1 %}",
                           "first",
                           "{% elsif a == 2 %}",
                           "second",
                           "{% elsif a == 3 %}",
                           "third",
                           "{% else %}",
                           "fourth",
                           "{% endif %}");

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("a", Value::scalar(1f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("first".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("a", Value::scalar(2f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("second".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("a", Value::scalar(3f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("third".to_owned()));

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("a", Value::scalar("else"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("fourth".to_owned()));
    }

    #[test]
    fn string_contains_with_literals() {
        let text = "{% if \"Star Wars\" contains \"Star\" %}if true{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));

        let text = "{% if \"Star Wars\" contains \"Alf\"  %}if true{% else %}if false{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }

    #[test]
    fn string_contains_with_variables() {
        let text = "{% if movie contains \"Star\"  %}if true{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("movie", Value::scalar("Star Wars"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));

        let text = "{% if movie contains \"Star\"  %}if true{% else %}if false{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_global_val("movie", Value::scalar("Batman"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }

    #[test]
    fn contains_with_object_and_key() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let mut obj = HashMap::new();
        obj.insert("Star Wars".to_owned(), Value::scalar("1977"));
        context.set_global_val("movies", Value::Object(obj));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));
    }

    #[test]
    fn contains_with_object_and_missing_key() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% else %}if false{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let obj = HashMap::new();
        context.set_global_val("movies", Value::Object(obj));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }

    #[test]
    fn contains_with_array_and_match() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let arr = vec![Value::scalar("Star Wars"),
                       Value::scalar("Star Trek"),
                       Value::scalar("Alien")];
        context.set_global_val("movies", Value::Array(arr));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if true".to_owned()));
    }

    #[test]
    fn contains_with_array_and_no_match() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% else %}if false{% endif %}";
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let arr = vec![Value::scalar("Alien")];
        context.set_global_val("movies", Value::Array(arr));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("if false".to_owned()));
    }
}
