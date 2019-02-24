use std::fmt;
use std::io::Write;

use itertools;
use liquid_error::{Error, Result, ResultLiquidExt, ResultLiquidReplaceExt};
use liquid_value::{Object, Scalar, Value};

use compiler::BlockElement;
use compiler::Language;
use compiler::TagBlock;
use compiler::TagTokenIter;
use compiler::TryMatchToken;
use interpreter::Expression;
use interpreter::Renderable;
use interpreter::Template;
use interpreter::{Context, Interrupt};

#[derive(Clone, Debug)]
enum Range {
    Array(Expression),
    Counted(Expression, Expression),
}

impl Range {
    pub fn evaluate(&self, context: &Context) -> Result<Vec<Value>> {
        let range = match *self {
            Range::Array(ref array_id) => get_array(context, array_id)?,

            Range::Counted(ref start_arg, ref stop_arg) => {
                let start = int_argument(start_arg, context, "start")?;
                let stop = int_argument(stop_arg, context, "end")?;
                let range = start..=stop;
                range.map(|x| Value::scalar(x as i32)).collect()
            }
        };

        Ok(range)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Range::Array(ref arr) => write!(f, "{}", arr),
            Range::Counted(ref start, ref end) => write!(f, "({}..{})", start, end),
        }
    }
}

fn iter_array(
    mut range: Vec<Value>,
    limit: Option<usize>,
    offset: usize,
    reversed: bool,
) -> Vec<Value> {
    let offset = ::std::cmp::min(offset, range.len());
    let limit = limit
        .map(|l| ::std::cmp::min(l, range.len()))
        .unwrap_or_else(|| range.len() - offset);
    range.drain(0..offset);
    range.resize(limit, Value::Nil);

    if reversed {
        range.reverse();
    };

    range
}

/// Extracts an integer value or an identifier from the token stream
fn parse_attr(arguments: &mut TagTokenIter) -> Result<Expression> {
    arguments
        .expect_next("\":\" expected.")?
        .expect_str(":")
        .into_result_custom_msg("\":\" expected.")?;

    arguments
        .expect_next("Value expected.")?
        .expect_value()
        .into_result()
}

/// Evaluates an attribute, returning Ok(None) if input is also None.
fn evaluate_attr(attr: &Option<Expression>, context: &mut Context) -> Result<Option<usize>> {
    match attr {
        Some(attr) => {
            let value = attr.evaluate(context)?;
            let value = value
                .as_scalar()
                .and_then(Scalar::to_integer)
                .ok_or_else(|| unexpected_value_error("whole number", Some(value.type_name())))?
                as usize;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

#[derive(Debug)]
struct For {
    var_name: String,
    range: Range,
    item_template: Template,
    else_template: Option<Template>,
    limit: Option<Expression>,
    offset: Option<Expression>,
    reversed: bool,
}

impl For {
    fn trace(&self) -> String {
        trace_for_tag(
            &self.var_name,
            &self.range,
            &self.limit,
            &self.offset,
            self.reversed,
        )
    }
}

fn get_array(context: &Context, array_id: &Expression) -> Result<Vec<Value>> {
    let array = array_id.evaluate(context)?;
    match array {
        Value::Empty => Ok(vec![]),
        Value::Array(x) => Ok(x.to_owned()),
        Value::Object(x) => {
            let x = x
                .iter()
                .map(|(k, v)| Value::Array(vec![Value::scalar(k.clone()), v.to_owned()]))
                .collect();
            Ok(x)
        }
        x => Err(unexpected_value_error("array", Some(x.type_name()))),
    }
}

fn int_argument(arg: &Expression, context: &Context, arg_name: &str) -> Result<isize> {
    let value = arg.evaluate(context)?;

    let value = value
        .as_scalar()
        .and_then(Scalar::to_integer)
        .ok_or_else(|| unexpected_value_error("whole number", Some(value.type_name())))
        .context_key_with(|| arg_name.to_owned().into())
        .value_with(|| value.to_str().into_owned().into())?;

    Ok(value as isize)
}

impl Renderable for For {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let range = self
            .range
            .evaluate(context)
            .trace_with(|| self.trace().into())?;
        let limit = evaluate_attr(&self.limit, context)?;
        let offset = evaluate_attr(&self.offset, context)?.unwrap_or(0);
        let range = iter_array(range, limit, offset, self.reversed);

        match range.len() {
            0 => {
                if let Some(ref t) = self.else_template {
                    t.render_to(writer, context)
                        .trace("{{% else %}}")
                        .trace_with(|| self.trace().into())?;
                }
            }

            range_len => {
                context.run_in_scope(|mut scope| -> Result<()> {
                    let mut helper_vars = Object::new();
                    helper_vars.insert("length".into(), Value::scalar(range_len as i32));

                    for (i, v) in range.into_iter().enumerate() {
                        helper_vars.insert("index0".into(), Value::scalar(i as i32));
                        helper_vars.insert("index".into(), Value::scalar((i + 1) as i32));
                        helper_vars
                            .insert("rindex0".into(), Value::scalar((range_len - i - 1) as i32));
                        helper_vars.insert("rindex".into(), Value::scalar((range_len - i) as i32));
                        helper_vars.insert("first".into(), Value::scalar(i == 0));
                        helper_vars.insert("last".into(), Value::scalar(i == (range_len - 1)));

                        scope
                            .stack_mut()
                            .set("forloop", Value::Object(helper_vars.clone()));
                        scope.stack_mut().set(self.var_name.to_owned(), v);
                        self.item_template
                            .render_to(writer, &mut scope)
                            .trace_with(|| self.trace().into())
                            .context_key("index")
                            .value_with(|| format!("{}", i + 1).into())?;

                        // given that we're at the end of the loop body
                        // already, dealing with a `continue` signal is just
                        // clearing the interrupt and carrying on as normal. A
                        // `break` requires some special handling, though.
                        if let Some(Interrupt::Break) = scope.interrupt_mut().pop_interrupt() {
                            break;
                        }
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

fn trace_for_tag(
    var_name: &str,
    range: &Range,
    limit: &Option<Expression>,
    offset: &Option<Expression>,
    reversed: bool,
) -> String {
    let mut parameters = vec![];
    if let Some(limit) = limit {
        parameters.push(format!("limit:{}", limit));
    }
    if let Some(offset) = offset {
        parameters.push(format!("offset:{}", offset));
    }
    if reversed {
        parameters.push("reversed".to_owned());
    }
    format!(
        "{{% for {} in {} {} %}}",
        var_name,
        range,
        itertools::join(parameters.iter(), ", ")
    )
}

pub fn for_block(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &Language,
) -> Result<Box<Renderable>> {
    let var_name = arguments
        .expect_next("Identifier expected.")?
        .expect_identifier()
        .into_result()?
        .to_string();

    arguments
        .expect_next("\"in\" expected.")?
        .expect_str("in")
        .into_result_custom_msg("\"in\" expected.")?;

    let range = arguments.expect_next("Array or range expected.")?;
    let range = match range.expect_value() {
        TryMatchToken::Matches(array) => Range::Array(array),
        TryMatchToken::Fails(range) => match range.expect_range() {
            TryMatchToken::Matches((start, stop)) => Range::Counted(start, stop),
            TryMatchToken::Fails(range) => return range.raise_error().into_err(),
        },
    };

    // now we get to check for parameters...
    let mut limit = None;
    let mut offset = None;
    let mut reversed = false;

    while let Some(token) = arguments.next() {
        match token.as_str() {
            "limit" => limit = Some(parse_attr(&mut arguments)?),
            "offset" => offset = Some(parse_attr(&mut arguments)?),
            "reversed" => reversed = true,
            _ => {
                return token
                    .raise_custom_error("\"limit\", \"offset\" or \"reversed\" expected.")
                    .into_err()
            }
        }
    }

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let mut item_template = Vec::new();
    let mut else_template = None;

    while let Some(element) = tokens.next()? {
        match element {
            BlockElement::Tag(mut tag) => match tag.name() {
                "else" => {
                    // no more arguments should be supplied, trying to supply them is an error
                    tag.tokens().expect_nothing()?;
                    else_template = Some(tokens.parse_all(options)?);
                    break;
                }
                _ => item_template.push(tag.parse(&mut tokens, options)?),
            },
            element => item_template.push(element.parse(&mut tokens, options)?),
        }
    }

    let item_template = Template::new(item_template);
    let else_template = else_template.map(Template::new);

    tokens.assert_empty();
    Ok(Box::new(For {
        var_name,
        range,
        item_template,
        else_template,
        limit,
        offset,
        reversed,
    }))
}

#[derive(Debug)]
struct TableRow {
    var_name: String,
    range: Range,
    item_template: Template,
    cols: Option<Expression>,
    limit: Option<Expression>,
    offset: Option<Expression>,
}

impl TableRow {
    fn trace(&self) -> String {
        trace_tablerow_tag(
            &self.var_name,
            &self.range,
            &self.cols,
            &self.limit,
            &self.offset,
        )
    }
}

fn trace_tablerow_tag(
    var_name: &str,
    range: &Range,
    cols: &Option<Expression>,
    limit: &Option<Expression>,
    offset: &Option<Expression>,
) -> String {
    let mut parameters = vec![];
    if let Some(cols) = cols {
        parameters.push(format!("cols:{}", cols));
    }
    if let Some(limit) = limit {
        parameters.push(format!("limit:{}", limit));
    }
    if let Some(offset) = offset {
        parameters.push(format!("offset:{}", offset));
    }
    format!(
        "{{% for {} in {} {} %}}",
        var_name,
        range,
        itertools::join(parameters.iter(), ", ")
    )
}

impl Renderable for TableRow {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let range = self
            .range
            .evaluate(context)
            .trace_with(|| self.trace().into())?;
        let cols = evaluate_attr(&self.cols, context)?;
        let limit = evaluate_attr(&self.limit, context)?;
        let offset = evaluate_attr(&self.offset, context)?.unwrap_or(0);
        let range = iter_array(range, limit, offset, false);

        context.run_in_scope(|mut scope| -> Result<()> {
            let mut helper_vars = Object::new();

            let range_len = range.len();
            helper_vars.insert("length".into(), Value::scalar(range_len as i32));

            for (i, v) in range.into_iter().enumerate() {
                let (col_index, row_index) = match cols {
                    Some(cols) => (i % cols, i / cols),
                    None => (i, 0),
                };

                let first = i == 0;
                let last = i == (range_len - 1);
                let col_first = col_index == 0;
                let col_last = cols.filter(|&cols| col_index + 1 == cols).is_some() || last;

                helper_vars.insert("index0".into(), Value::scalar(i as i32));
                helper_vars.insert("index".into(), Value::scalar((i + 1) as i32));
                helper_vars.insert("rindex0".into(), Value::scalar((range_len - i - 1) as i32));
                helper_vars.insert("rindex".into(), Value::scalar((range_len - i) as i32));
                helper_vars.insert("first".into(), Value::scalar(first));
                helper_vars.insert("last".into(), Value::scalar(last));
                helper_vars.insert("col0".into(), Value::scalar(col_index as i32));
                helper_vars.insert("col".into(), Value::scalar((col_index + 1) as i32));
                helper_vars.insert("col_first".into(), Value::scalar(col_first));
                helper_vars.insert("col_last".into(), Value::scalar(col_last));
                scope
                    .stack_mut()
                    .set("tablerow", Value::Object(helper_vars.clone()));

                if col_first {
                    write!(writer, "<tr class=\"row{}\">", row_index + 1)
                        .replace("Failed to render")?;
                }
                write!(writer, "<td class=\"col{}\">", col_index + 1)
                    .replace("Failed to render")?;

                scope.stack_mut().set(self.var_name.to_owned(), v);
                self.item_template
                    .render_to(writer, &mut scope)
                    .trace_with(|| self.trace().into())
                    .context_key("index")
                    .value_with(|| format!("{}", i + 1).into())?;

                write!(writer, "</td>").replace("Failed to render")?;
                if col_last {
                    write!(writer, "</tr>").replace("Failed to render")?;
                }
            }
            Ok(())
        })?;

        Ok(())
    }
}

pub fn tablerow_block(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &Language,
) -> Result<Box<Renderable>> {
    let var_name = arguments
        .expect_next("Identifier expected.")?
        .expect_identifier()
        .into_result()?
        .to_string();

    arguments
        .expect_next("\"in\" expected.")?
        .expect_str("in")
        .into_result_custom_msg("\"in\" expected.")?;

    let range = arguments.expect_next("Array or range expected.")?;
    let range = match range.expect_value() {
        TryMatchToken::Matches(array) => Range::Array(array),
        TryMatchToken::Fails(range) => match range.expect_range() {
            TryMatchToken::Matches((start, stop)) => Range::Counted(start, stop),
            TryMatchToken::Fails(range) => return range.raise_error().into_err(),
        },
    };

    // now we get to check for parameters...
    let mut cols = None;
    let mut limit = None;
    let mut offset = None;

    while let Some(token) = arguments.next() {
        match token.as_str() {
            "cols" => cols = Some(parse_attr(&mut arguments)?),
            "limit" => limit = Some(parse_attr(&mut arguments)?),
            "offset" => offset = Some(parse_attr(&mut arguments)?),
            _ => {
                return token
                    .raise_custom_error("\"cols\", \"limit\" or \"offset\" expected.")
                    .into_err()
            }
        }
    }

    // no more arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let item_template = Template::new(tokens.parse_all(options)?);

    tokens.assert_empty();
    Ok(Box::new(TableRow {
        var_name,
        range,
        item_template,
        cols,
        limit,
        offset,
    }))
}

/// Format an error for an unexpected value.
pub fn unexpected_value_error<S: ToString>(expected: &str, actual: Option<S>) -> Error {
    let actual = actual.map(|x| x.to_string());
    unexpected_value_error_string(expected, actual)
}

fn unexpected_value_error_string(expected: &str, actual: Option<String>) -> Error {
    let actual = actual.unwrap_or_else(|| "nothing".to_owned());
    Error::with_msg(format!("Expected {}, found `{}`", expected, actual))
}

#[cfg(test)]
mod test {
    use compiler;
    use compiler::Filter;
    use derive::*;
    use interpreter;
    use interpreter::ContextBuilder;
    use tags;

    use super::*;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .blocks
            .register("for", (for_block as compiler::FnParseBlock).into());
        options.blocks.register(
            "tablerow",
            (tablerow_block as compiler::FnParseBlock).into(),
        );
        options
            .tags
            .register("assign", (tags::assign_tag as compiler::FnParseTag).into());
        options
    }

    #[test]
    fn loop_over_array() {
        let text = concat!("{% for name in array %}", "test {{name}} ", "{% endfor %}",);

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context: Context = Default::default();
        context.stack_mut().set_global(
            "array",
            Value::Array(vec![
                Value::scalar(22f64),
                Value::scalar(23f64),
                Value::scalar(24f64),
                Value::scalar("wat".to_owned()),
            ]),
        );
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "test 22 test 23 test 24 test wat ");
    }

    #[test]
    fn loop_over_range_literals() {
        let text = concat!(
            "{% for name in (42..46) %}",
            "#{{forloop.index}} test {{name}} | ",
            "{% endfor %}",
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Default::default();
        let output = template.render(&mut context).unwrap();
        assert_eq!(
            output,
            "#1 test 42 | #2 test 43 | #3 test 44 | #4 test 45 | #5 test 46 | "
        );
    }

    #[test]
    fn loop_over_range_vars() {
        let text = concat!(
            "{% for x in (alpha .. omega) %}",
            "#{{forloop.index}} test {{x}}, ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context
            .stack_mut()
            .set_global("alpha", Value::scalar(42i32));
        context
            .stack_mut()
            .set_global("omega", Value::scalar(46i32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(
            output,
            "#1 test 42, #2 test 43, #3 test 44, #4 test 45, #5 test 46, "
        );
    }

    #[test]
    fn nested_forloops() {
        // test that nest nested for loops work, and that the
        // variable scopes between the inner and outer variable
        // scopes do not overlap.
        let text = concat!(
            "{% for outer in (1..5) %}",
            ">>{{forloop.index0}}:{{outer}}>>",
            "{% for inner in (6..10) %}",
            "{{outer}}:{{forloop.index0}}:{{inner}},",
            "{% endfor %}",
            ">>{{outer}}>>\n",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(
            output,
            concat!(
                ">>0:1>>1:0:6,1:1:7,1:2:8,1:3:9,1:4:10,>>1>>\n",
                ">>1:2>>2:0:6,2:1:7,2:2:8,2:3:9,2:4:10,>>2>>\n",
                ">>2:3>>3:0:6,3:1:7,3:2:8,3:3:9,3:4:10,>>3>>\n",
                ">>3:4>>4:0:6,4:1:7,4:2:8,4:3:9,4:4:10,>>4>>\n",
                ">>4:5>>5:0:6,5:1:7,5:2:8,5:3:9,5:4:10,>>5>>\n",
            )
        );
    }

    #[test]
    fn nested_forloops_with_else() {
        // test that nested for loops parse their `else` blocks correctly
        let text = concat!(
            "{% for x in i %}",
            "{% for y in j %}inner{% else %}empty inner{% endfor %}",
            "{% else %}",
            "empty outer",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.stack_mut().set_global("i", Value::Array(vec![]));
        context.stack_mut().set_global("j", Value::Array(vec![]));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "empty outer");

        context
            .stack_mut()
            .set_global("i", Value::Array(vec![Value::scalar(1i32)]));
        context.stack_mut().set_global("j", Value::Array(vec![]));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "empty inner");
    }

    #[test]
    fn degenerate_range_is_safe() {
        // make sure that a degenerate range (i.e. where max < min)
        // doesn't result in an infinte loop
        let text = concat!("{% for x in (10 .. 0) %}", "{{x}}", "{% endfor %}");
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn limited_loop() {
        let text = concat!(
            "{% for i in (1..100) limit:2 %}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "1 2 ");
    }

    #[test]
    fn offset_loop() {
        let text = concat!(
            "{% for i in (1..10) offset:4 %}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "5 6 7 8 9 10 ");
    }

    #[test]
    fn offset_and_limited_loop() {
        let text = concat!(
            "{% for i in (1..10) offset:4 limit:2 %}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "5 6 ");
    }

    #[test]
    fn reversed_loop() {
        let text = concat!(
            "{% for i in (1..10) reversed %}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "10 9 8 7 6 5 4 3 2 1 ");
    }

    #[test]
    fn sliced_and_reversed_loop() {
        let text = concat!(
            "{% for i in (1..10) reversed offset:1 limit:5%}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "6 5 4 3 2 ");
    }

    #[test]
    fn empty_loop_invokes_else_template() {
        let text = concat!(
            "{% for i in (1..10) limit:0 %}",
            "{{ i }} ",
            "{% else %}",
            "There are no items!",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "There are no items!");
    }

    #[test]
    fn limit_greater_than_iterator_length() {
        let text = concat!("{% for i in (1..5) limit:10 %}", "{{ i }} ", "{% endfor %}");
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "1 2 3 4 5 ");
    }

    #[test]
    fn loop_variables() {
        let text = concat!(
            "{% for v in (100..102) %}",
            "length: {{forloop.length}}, ",
            "index: {{forloop.index}}, ",
            "index0: {{forloop.index0}}, ",
            "rindex: {{forloop.rindex}}, ",
            "rindex0: {{forloop.rindex0}}, ",
            "value: {{v}}, ",
            "first: {{forloop.first}}, ",
            "last: {{forloop.last}}\n",
            "{% endfor %}",
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context: Context = Default::default();
        let output = template.render(&mut context).unwrap();
        assert_eq!(
                output,
                concat!(
    "length: 3, index: 1, index0: 0, rindex: 3, rindex0: 2, value: 100, first: true, last: false\n",
    "length: 3, index: 2, index0: 1, rindex: 2, rindex0: 1, value: 101, first: false, last: false\n",
    "length: 3, index: 3, index0: 2, rindex: 1, rindex0: 0, value: 102, first: false, last: true\n",
    )
            );
    }

    #[derive(Clone, ParseFilter, FilterReflection)]
    #[filter(name = "shout", description = "tests helper", parsed(ShoutFilter))]
    pub struct ShoutFilterParser;

    #[derive(Debug, Default, Display_filter)]
    #[name = "shout"]
    pub struct ShoutFilter;

    impl Filter for ShoutFilter {
        fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
            Ok(Value::scalar(input.to_str().to_uppercase()))
        }
    }

    #[test]
    fn use_filters() {
        let text = concat!(
            "{% for name in array %}",
            "test {{name | shout}} ",
            "{% endfor %}",
        );

        let mut options = options();
        options
            .filters
            .register("shout", Box::new(ShoutFilterParser));
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = ContextBuilder::new().build();

        context.stack_mut().set_global(
            "array",
            Value::Array(vec![
                Value::scalar("alpha"),
                Value::scalar("beta"),
                Value::scalar("gamma"),
            ]),
        );
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "test ALPHA test BETA test GAMMA ");
    }

    #[test]
    fn for_loop_parameters_with_variables() {
        let text = concat!(
            "{% assign l = 4 %}",
            "{% assign o = 5 %}",
            "{% for i in (1..100) limit:l offset:o %}",
            "{{ i }} ",
            "{% endfor %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "6 7 8 9 ");
    }

    #[test]
    fn tablerow_without_cols() {
        let text = concat!(
            "{% tablerow name in array %}",
            "test {{name}} ",
            "{% endtablerow %}",
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context: Context = Default::default();
        context.stack_mut().set_global(
            "array",
            Value::Array(vec![
                Value::scalar(22f64),
                Value::scalar(23f64),
                Value::scalar(24f64),
                Value::scalar("wat".to_owned()),
            ]),
        );
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "<tr class=\"row1\"><td class=\"col1\">test 22 </td><td class=\"col2\">test 23 </td><td class=\"col3\">test 24 </td><td class=\"col4\">test wat </td></tr>");
    }

    #[test]
    fn tablerow_with_cols() {
        let text = concat!(
            "{% tablerow name in (42..46) cols:2 %}",
            "test {{name}} ",
            "{% endtablerow %}",
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context: Context = Default::default();
        context.stack_mut().set_global(
            "array",
            Value::Array(vec![
                Value::scalar(22f64),
                Value::scalar(23f64),
                Value::scalar(24f64),
                Value::scalar("wat".to_owned()),
            ]),
        );
        let output = template.render(&mut context).unwrap();
        assert_eq!(
                output,
                "<tr class=\"row1\"><td class=\"col1\">test 42 </td><td class=\"col2\">test 43 </td></tr><tr class=\"row2\"><td class=\"col1\">test 44 </td><td class=\"col2\">test 45 </td></tr><tr class=\"row3\"><td class=\"col1\">test 46 </td></tr>"
            );
    }

    #[test]
    fn tablerow_loop_parameters_with_variables() {
        let text = concat!(
            "{% assign l = 4 %}",
            "{% assign o = 5 %}",
            "{% assign c = 3 %}",
            "{% tablerow i in (1..100) limit:l offset:o cols:c %}",
            "{{ i }} ",
            "{% endtablerow %}"
        );
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "<tr class=\"row1\"><td class=\"col1\">6 </td><td class=\"col2\">7 </td><td class=\"col3\">8 </td></tr><tr class=\"row2\"><td class=\"col1\">9 </td></tr>");
    }

    #[test]
    fn tablerow_variables() {
        let text = concat!(
            "{% tablerow v in (100..103) cols:2 %}",
            "length: {{tablerow.length}}, ",
            "index: {{tablerow.index}}, ",
            "index0: {{tablerow.index0}}, ",
            "rindex: {{tablerow.rindex}}, ",
            "rindex0: {{tablerow.rindex0}}, ",
            "col: {{tablerow.col}}, ",
            "col0: {{tablerow.col0}}, ",
            "value: {{v}}, ",
            "first: {{tablerow.first}}, ",
            "last: {{tablerow.last}}, ",
            "col_first: {{tablerow.col_first}}, ",
            "col_last: {{tablerow.col_last}}",
            "{% endtablerow %}",
        );

        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context: Context = Default::default();
        let output = template.render(&mut context).unwrap();
        assert_eq!(
                output,
                concat!(
    "<tr class=\"row1\"><td class=\"col1\">length: 4, index: 1, index0: 0, rindex: 4, rindex0: 3, col: 1, col0: 0, value: 100, first: true, last: false, col_first: true, col_last: false</td>",
    "<td class=\"col2\">length: 4, index: 2, index0: 1, rindex: 3, rindex0: 2, col: 2, col0: 1, value: 101, first: false, last: false, col_first: false, col_last: true</td></tr>",
    "<tr class=\"row2\"><td class=\"col1\">length: 4, index: 3, index0: 2, rindex: 2, rindex0: 1, col: 1, col0: 0, value: 102, first: false, last: false, col_first: true, col_last: false</td>",
    "<td class=\"col2\">length: 4, index: 4, index0: 3, rindex: 1, rindex0: 0, col: 2, col0: 1, value: 103, first: false, last: true, col_first: false, col_last: true</td></tr>",
    )
            );
    }
}
