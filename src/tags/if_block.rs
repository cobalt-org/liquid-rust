use std::fmt;
use std::io::Write;

use liquid_error::{Result, ResultLiquidExt};
use liquid_value::Value;

use compiler::BlockElement;
use compiler::LiquidOptions;
use compiler::TagBlock;
use compiler::TagToken;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;
use interpreter::{unexpected_value_error, Expression};

#[derive(Clone, Debug)]
enum ComparisonOperator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Contains,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            ComparisonOperator::Equals => "==",
            ComparisonOperator::NotEquals => "!=",
            ComparisonOperator::LessThanEquals => "<=",
            ComparisonOperator::GreaterThanEquals => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::Contains => "contains",
        };
        write!(f, "{}", out)
    }
}

impl ComparisonOperator {
    fn from_str(s: &str) -> ::std::result::Result<Self, ()> {
        match s {
            "==" => Ok(ComparisonOperator::Equals),
            "!=" | "<>" => Ok(ComparisonOperator::NotEquals),
            "<" => Ok(ComparisonOperator::LessThan),
            ">" => Ok(ComparisonOperator::GreaterThan),
            "<=" => Ok(ComparisonOperator::LessThanEquals),
            ">=" => Ok(ComparisonOperator::GreaterThanEquals),
            "contains" => Ok(ComparisonOperator::Contains),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
struct BinaryCondition {
    lh: Expression,
    comparison: ComparisonOperator,
    rh: Expression,
}

impl BinaryCondition {
    pub fn evaluate(&self, context: &Context) -> Result<bool> {
        let a = self.lh.evaluate(context)?;
        let b = self.rh.evaluate(context)?;

        let result = match self.comparison {
            ComparisonOperator::Equals => a == b,
            ComparisonOperator::NotEquals => a != b,
            ComparisonOperator::LessThan => a < b,
            ComparisonOperator::GreaterThan => a > b,
            ComparisonOperator::LessThanEquals => a <= b,
            ComparisonOperator::GreaterThanEquals => a >= b,
            ComparisonOperator::Contains => contains_check(&a, &b)?,
        };

        Ok(result)
    }
}

impl fmt::Display for BinaryCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.lh, self.comparison, self.rh)
    }
}

#[derive(Clone, Debug)]
struct ExistenceCondition {
    lh: Expression,
}

impl ExistenceCondition {
    pub fn evaluate(&self, context: &Context) -> Result<bool> {
        let a = self.lh.try_evaluate(context).cloned().unwrap_or_default();
        Ok(a.is_truthy())
    }
}

impl fmt::Display for ExistenceCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.lh)
    }
}

#[derive(Clone, Debug)]
enum Condition {
    Binary(BinaryCondition),
    Existence(ExistenceCondition),
    Conjunction(Box<Condition>, Box<Condition>),
    Disjunction(Box<Condition>, Box<Condition>),
}

impl Condition {
    pub fn evaluate(&self, context: &Context) -> Result<bool> {
        match *self {
            Condition::Binary(ref c) => c.evaluate(context),
            Condition::Existence(ref c) => c.evaluate(context),
            Condition::Conjunction(ref left, ref right) => {
                Ok(left.evaluate(context)? && right.evaluate(context)?)
            }
            Condition::Disjunction(ref left, ref right) => {
                Ok(left.evaluate(context)? || right.evaluate(context)?)
            }
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Condition::Binary(ref c) => write!(f, "{}", c),
            Condition::Existence(ref c) => write!(f, "{}", c),
            Condition::Conjunction(ref left, ref right) => write!(f, "{} and {}", left, right),
            Condition::Disjunction(ref left, ref right) => write!(f, "{} or {}", left, right),
        }
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
    match *a {
        Value::Scalar(ref val) => {
            let b = b.to_str();
            Ok(val.to_str().contains(b.as_ref()))
        }
        Value::Object(_) => {
            let b = b.as_scalar();
            let check = b.map(|b| a.contains_key(b)).unwrap_or(false);
            Ok(check)
        }
        Value::Array(ref arr) => {
            for elem in arr {
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
        let result = self.condition.evaluate(context)?;

        Ok(result == self.mode)
    }

    fn trace(&self) -> String {
        format!("{{% if {} %}}", self.condition)
    }
}

impl Renderable for Conditional {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let condition = self.compare(context).trace_with(|| self.trace())?;
        if condition {
            self.if_true
                .render_to(writer, context)
                .trace_with(|| self.trace())?;
        } else if let Some(ref template) = self.if_false {
            template
                .render_to(writer, context)
                .trace("{{% else %}}")
                .trace_with(|| self.trace())?;
        }

        Ok(())
    }
}

struct PeekableTagTokenIter<'a> {
    iter: TagTokenIter<'a>,
    peeked: Option<Option<TagToken<'a>>>,
}

impl<'a> Iterator for PeekableTagTokenIter<'a> {
    type Item = TagToken<'a>;

    fn next(&mut self) -> Option<TagToken<'a>> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        }
    }
}

impl<'a> PeekableTagTokenIter<'a> {
    pub fn expect_next(&mut self, error_msg: &str) -> Result<TagToken<'a>> {
        self.next().ok_or_else(|| self.iter.raise_error(error_msg))
    }

    fn peek(&mut self) -> Option<&TagToken<'a>> {
        if self.peeked.is_none() {
            self.peeked = Some(self.iter.next());
        }
        match self.peeked {
            Some(Some(ref value)) => Some(value),
            Some(None) => None,
            None => unreachable!(),
        }
    }
}

fn parse_atom_condition(arguments: &mut PeekableTagTokenIter) -> Result<Condition> {
    let lh = arguments
        .expect_next("Value expected.")?
        .expect_value()
        .into_result()?;
    let cond = match arguments
        .peek()
        .map(TagToken::as_str)
        .and_then(|op| ComparisonOperator::from_str(op).ok())
    {
        Some(op) => {
            arguments.next();
            let rh = arguments
                .expect_next("Value expected.")?
                .expect_value()
                .into_result()?;
            Condition::Binary(BinaryCondition {
                lh,
                comparison: op,
                rh,
            })
        }
        None => Condition::Existence(ExistenceCondition { lh }),
    };

    Ok(cond)
}

fn parse_conjunction_chain(arguments: &mut PeekableTagTokenIter) -> Result<Condition> {
    let mut lh = parse_atom_condition(arguments)?;

    while let Some("and") = arguments.peek().map(TagToken::as_str) {
        arguments.next();
        let rh = parse_atom_condition(arguments)?;
        lh = Condition::Conjunction(Box::new(lh), Box::new(rh));
    }

    Ok(lh)
}

/// Common parsing for "if" and "unless" condition
fn parse_condition(arguments: TagTokenIter) -> Result<Condition> {
    let mut arguments = PeekableTagTokenIter {
        iter: arguments,
        peeked: None,
    };
    let mut lh = parse_conjunction_chain(&mut arguments)?;

    while let Some(token) = arguments.next() {
        token
            .expect_str("or")
            .into_result_custom_msg("\"and\" or \"or\" expected.")?;

        let rh = parse_conjunction_chain(&mut arguments)?;
        lh = Condition::Disjunction(Box::new(lh), Box::new(rh));
    }

    Ok(lh)
}

pub fn unless_block(
    _tag_name: &str,
    arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let condition = parse_condition(arguments)?;
    let if_true = Template::new(tokens.parse_all(options)?);

    tokens.assert_empty();
    Ok(Box::new(Conditional {
        tag_name: "unless",
        condition,
        mode: false,
        if_true,
        if_false: None,
    }))
}

fn parse_if(
    tag_name: &'static str,
    arguments: TagTokenIter,
    tokens: &mut TagBlock,
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let condition = parse_condition(arguments)?;

    let mut if_true = Vec::new();
    let mut if_false = None;

    while let Some(element) = tokens.next()? {
        match element {
            BlockElement::Tag(mut tag) => match tag.name() {
                "else" => {
                    if_false = Some(tokens.parse_all(options)?);
                    break;
                }
                "elsif" => {
                    if_false = Some(vec![parse_if("elsif", tag.into_tokens(), tokens, options)?]);
                    break;
                }
                _ => if_true.push(tag.parse(tokens, options)?),
            },
            element => if_true.push(element.parse(tokens, options)?),
        }
    }

    let if_true = Template::new(if_true);
    let if_false = if_false.map(Template::new);

    Ok(Box::new(Conditional {
        tag_name,
        condition,
        mode: true,
        if_true,
        if_false,
    }))
}

pub fn if_block(
    _tag_name: &str,
    arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let conditional = parse_if("if", arguments, &mut tokens, options)?;

    tokens.assert_empty();
    Ok(conditional)
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use value::Object;
    use value::Value;

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
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = "{% if 7 < 6  %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn string_comparison() {
        let text = r#"{% if "one" == "one"  %}if true{% endif %}"#;
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = r#"{% if "one" == "two"  %}if true{% else %}if false{% endif %}"#;
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
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

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        // Non-existence
        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "nope");

        // Explicit nil
        let mut context = Context::new();
        context.stack_mut().set_global("truthy", Value::Nil);
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "nope");

        // false
        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("truthy", Value::scalar(false));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "nope");

        // true
        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("truthy", Value::scalar(true));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "yep");
    }

    #[test]
    fn unless() {
        let text = concat!(
            "{% unless some_value == 1 %}",
            "unless body",
            "{% endunless %}"
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("some_value", Value::scalar(1f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "");

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("some_value", Value::scalar(42f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "unless body");
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
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("truthy", Value::scalar(true));
        context
            .stack_mut()
            .set_global("also_truthy", Value::scalar(false));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "yep, not also truthy");
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

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global("a", Value::scalar(1f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "first");

        let mut context = Context::new();
        context.stack_mut().set_global("a", Value::scalar(2f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "second");

        let mut context = Context::new();
        context.stack_mut().set_global("a", Value::scalar(3f64));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "third");

        let mut context = Context::new();
        context.stack_mut().set_global("a", Value::scalar("else"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "fourth");
    }

    #[test]
    fn string_contains_with_literals() {
        let text = "{% if \"Star Wars\" contains \"Star\" %}if true{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = "{% if \"Star Wars\" contains \"Alf\"  %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn string_contains_with_variables() {
        let text = "{% if movie contains \"Star\"  %}if true{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("movie", Value::scalar("Star Wars"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = "{% if movie contains \"Star\"  %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("movie", Value::scalar("Batman"));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn contains_with_object_and_key() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let mut obj = Object::new();
        obj.insert("Star Wars".into(), Value::scalar("1977"));
        context.stack_mut().set_global("movies", Value::Object(obj));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");
    }

    #[test]
    fn contains_with_object_and_missing_key() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let obj = Object::new();
        context.stack_mut().set_global("movies", Value::Object(obj));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn contains_with_array_and_match() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let arr = vec![
            Value::scalar("Star Wars"),
            Value::scalar("Star Trek"),
            Value::scalar("Alien"),
        ];
        context.stack_mut().set_global("movies", Value::Array(arr));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");
    }

    #[test]
    fn contains_with_array_and_no_match() {
        let text = "{% if movies contains \"Star Wars\" %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let arr = vec![Value::scalar("Alien")];
        context.stack_mut().set_global("movies", Value::Array(arr));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn multiple_conditions_and() {
        let text = "{% if 1 == 1 and 2 == 2 %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = "{% if 1 == 1 and 2 != 2 %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn multiple_conditions_or() {
        let text = "{% if 1 == 1 or 2 != 2 %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");

        let text = "{% if 1 != 1 or 2 != 2 %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if false");
    }

    #[test]
    fn multiple_conditions_and_or() {
        let text = "{% if 1 == 1 or 2 == 2 and 3 != 3 %}if true{% else %}if false{% endif %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "if true");
    }
}
