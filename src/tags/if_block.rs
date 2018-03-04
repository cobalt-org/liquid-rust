use std::fmt;

use error::{Result, ResultLiquidExt};

use compiler::ComparisonOperator;
use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use compiler::{consume_value_token, parse, split_block, unexpected_token_error};
use interpreter::{unexpected_value_error, Argument};
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;
use value::Value;

#[derive(Clone, Debug)]
struct Condition {
    lh: Argument,
    comparison: ComparisonOperator,
    rh: Argument,
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.lh, self.comparison, self.rh)
    }
}

#[derive(Debug)]
struct Conditional {
    tag_name: &'static str,
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
        _ => Err(unexpected_value_error(
            "string | array | object",
            Some(a.type_name()),
        )),
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

    fn trace(&self) -> String {
        format!("{{% if {} %}}", self.condition)
    }
}

impl Renderable for Conditional {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let condition = self.compare(context).trace_with(|| self.trace().into());
        if condition? {
            self.if_true
                .render(context)
                .trace_with(|| self.trace().into())
        } else {
            match self.if_false {
                Some(ref template) => template
                    .render(context)
                    .trace_with(|| "{{% else %}}".to_owned().into())
                    .trace_with(|| self.trace().into()),
                _ => Ok(None),
            }
        }
    }
}

/// Common parsing for "if" and "unless" condition
fn parse_condition(arguments: &[Token]) -> Result<Condition> {
    let mut args = arguments.iter();

    let lh = consume_value_token(&mut args)?.to_arg()?;

    let (comp, rh) = match args.next() {
        Some(&Token::Comparison(x)) => {
            let rhs = consume_value_token(&mut args)?.to_arg()?;
            (x, rhs)
        }
        None => {
            // no trailing operator or RHS value implies "== true"
            (
                ComparisonOperator::Equals,
                Token::BooleanLiteral(true).to_arg()?,
            )
        }
        x @ Some(_) => return Err(unexpected_token_error("comparison operator", x)),
    };

    Ok(Condition {
        lh: lh,
        comparison: comp,
        rh: rh,
    })
}

pub fn unless_block(
    _tag_name: &str,
    arguments: &[Token],
    tokens: &[Element],
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let condition = parse_condition(arguments)?;
    let if_true = Template::new(parse(&tokens[..], options)?);
    Ok(Box::new(Conditional {
        tag_name: "unless",
        condition,
        mode: false,
        if_true,
        if_false: None,
    }))
}

pub fn if_block(
    _tag_name: &str,
    arguments: &[Token],
    tokens: &[Element],
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let condition = parse_condition(arguments)?;

    let (leading_tokens, trailing_tokens) = split_block(&tokens[..], &["else", "elsif"], options);

    let if_true =
        parse(leading_tokens, options).trace_with(|| format!("{{% if {} %}}", condition).into())?;
    let if_true = Template::new(if_true);

    let if_false = match trailing_tokens {
        None => Ok(None),

        Some(ref split) if split.delimiter == "else" => parse(&split.trailing[1..], options)
            .map(Some)
            .trace_with(|| "{{% else %}}".to_owned().into()),

        Some(ref split) if split.delimiter == "elsif" => {
            let child_tokens: Vec<Element> = split.trailing.iter().skip(1).cloned().collect();
            if_block("elseif", &split.args[1..], &child_tokens, options)
                .map(|block| Some(vec![block]))
        }

        Some(split) => panic!("Unexpected delimiter: {:?}", split.delimiter),
    };
    let if_false = if_false
        .trace_with(|| format!("{{% if {} %}}", condition).into())?
        .map(Template::new);

    Ok(Box::new(Conditional {
        tag_name: "if",
        condition,
        mode: true,
        if_true,
        if_false,
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use value::Value;
    use value::Object;
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
        let text = concat!(
            "{% if truthy %}",
            "yep",
            "{% else %}",
            "nope",
            "{% endif %}"
        );

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
        let text = concat!(
            "{% unless some_value == 1 %}",
            "unless body",
            "{% endunless %}"
        );

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
        let text = concat!(
            "{% if truthy %}",
            "yep, ",
            "{% if also_truthy %}",
            "also truthy",
            "{% else %}",
            "not also truthy",
            "{% endif %}",
            "{% else %}",
            "nope",
            "{% endif %}"
        );
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
        let text = concat!(
            "{% if a == 1 %}",
            "first",
            "{% elsif a == 2 %}",
            "second",
            "{% elsif a == 3 %}",
            "third",
            "{% else %}",
            "fourth",
            "{% endif %}"
        );

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
        let mut obj = Object::new();
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
        let obj = Object::new();
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
        let arr = vec![
            Value::scalar("Star Wars"),
            Value::scalar("Star Trek"),
            Value::scalar("Alien"),
        ];
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
