use std::io::Write;

use itertools;
use liquid_error::{Result, ResultLiquidExt};
use liquid_value::Value;

use compiler::BlockElement;
use compiler::Language;
use compiler::TagBlock;
use compiler::TagTokenIter;
use compiler::TryMatchToken;
use interpreter::Context;
use interpreter::Expression;
use interpreter::Renderable;
use interpreter::Template;

#[derive(Debug)]
struct CaseOption {
    args: Vec<Expression>,
    template: Template,
}

impl CaseOption {
    fn new(args: Vec<Expression>, template: Template) -> CaseOption {
        CaseOption { args, template }
    }

    fn evaluate(&self, value: &Value, context: &Context) -> Result<bool> {
        for a in &self.args {
            let v = a.evaluate(context)?;
            if *v == *value {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn trace(&self) -> String {
        format!("{{% when {} %}}", itertools::join(self.args.iter(), " or "))
    }
}

#[derive(Debug)]
struct Case {
    target: Expression,
    cases: Vec<CaseOption>,
    else_block: Option<Template>,
}

impl Case {
    fn trace(&self) -> String {
        format!("{{% case {} %}}", self.target)
    }
}

impl Renderable for Case {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let value = self.target.evaluate(context)?.to_owned();
        for case in &self.cases {
            if case.evaluate(&value, context)? {
                return case
                    .template
                    .render_to(writer, context)
                    .trace_with(|| case.trace().into())
                    .trace_with(|| self.trace().into())
                    .context_key_with(|| self.target.to_string().into())
                    .value_with(|| value.to_str().into_owned().into());
            }
        }

        if let Some(ref t) = self.else_block {
            return t
                .render_to(writer, context)
                .trace("{{% else %}}")
                .trace_with(|| self.trace().into())
                .context_key_with(|| self.target.to_string().into())
                .value_with(|| value.to_str().into_owned().into());
        }

        Ok(())
    }
}

fn parse_condition(arguments: &mut TagTokenIter) -> Result<Vec<Expression>> {
    let mut values = Vec::new();

    let first_value = arguments
        .expect_next("Value expected")?
        .expect_value()
        .into_result()?;
    values.push(first_value);

    while let Some(token) = arguments.next() {
        if let TryMatchToken::Fails(token) = token.expect_str("or") {
            token
                .expect_str(",")
                .into_result_custom_msg("\"or\" or \",\" expected.")?;
        }

        let value = arguments
            .expect_next("Value expected")?
            .expect_value()
            .into_result()?;
        values.push(value);
    }

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;
    Ok(values)
}

pub fn case_block(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &Language,
) -> Result<Box<Renderable>> {
    let target = arguments
        .expect_next("Value expected.")?
        .expect_value()
        .into_result()?;

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let mut cases = Vec::new();
    let mut else_block = None;
    let mut current_block = Vec::new();
    let mut current_condition = None;

    while let Some(element) = tokens.next()? {
        match element {
            BlockElement::Tag(mut tag) => match tag.name() {
                "when" => {
                    if let Some(condition) = current_condition {
                        cases.push(CaseOption::new(condition, Template::new(current_block)));
                    }
                    current_block = Vec::new();
                    current_condition = Some(parse_condition(tag.tokens())?);
                }
                "else" => {
                    // no more arguments should be supplied, trying to supply them is an error
                    tag.tokens().expect_nothing()?;
                    else_block = Some(tokens.parse_all(options)?);
                    break;
                }
                _ => current_block.push(tag.parse(&mut tokens, options)?),
            },
            element => current_block.push(element.parse(&mut tokens, options)?),
        }
    }

    if let Some(condition) = current_condition {
        cases.push(CaseOption::new(condition, Template::new(current_block)));
    }

    let else_block = else_block.map(Template::new);

    tokens.assert_empty();
    Ok(Box::new(Case {
        target,
        cases,
        else_block,
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .blocks
            .register("case", (case_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn test_case_block() {
        let text = concat!(
            "{% case x %}",
            "{% when 2 %}",
            "two",
            "{% when 3 or 4 %}",
            "three and a half",
            "{% else %}",
            "otherwise",
            "{% endcase %}"
        );
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global("x", Value::scalar(2f64));
        assert_eq!(template.render(&mut context).unwrap(), "two");

        context.stack_mut().set_global("x", Value::scalar(3f64));
        assert_eq!(template.render(&mut context).unwrap(), "three and a half");

        context.stack_mut().set_global("x", Value::scalar(4f64));
        assert_eq!(template.render(&mut context).unwrap(), "three and a half");

        context.stack_mut().set_global("x", Value::scalar("nope"));
        assert_eq!(template.render(&mut context).unwrap(), "otherwise");
    }

    #[test]
    fn test_no_matches_returns_empty_string() {
        let text = concat!(
            "{% case x %}",
            "{% when 2 %}",
            "two",
            "{% when 3 or 4 %}",
            "three and a half",
            "{% endcase %}"
        );
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global("x", Value::scalar("nope"));
        assert_eq!(template.render(&mut context).unwrap(), "");
    }

    #[test]
    fn multiple_else_blocks_is_an_error() {
        let text = concat!(
            "{% case x %}",
            "{% when 2 %}",
            "two",
            "{% else %}",
            "else #1",
            "{% else %}",
            "else # 2",
            "{% endcase %}"
        );
        let options = options();
        let template = compiler::parse(text, &options).map(interpreter::Template::new);
        assert!(template.is_err());
    }
}
