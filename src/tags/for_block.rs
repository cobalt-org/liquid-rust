use std::collections::HashMap;
use std::slice::Iter;

use error::{Error, Result};

use interpreter::Argument;
use interpreter::Renderable;
use interpreter::Template;
use interpreter::{Context, Interrupt};
use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use compiler::{parse, expect, split_block};
use value::Value;

#[derive(Clone, Debug)]
enum Range {
    Array(String),
    Counted(Argument, Argument),
}

#[derive(Debug)]
struct For {
    var_name: String,
    range: Range,
    item_template: Template,
    else_template: Option<Template>,
    limit: Option<usize>,
    offset: usize,
    reversed: bool,
}

fn get_array(context: &Context, array_id: &str) -> Result<Vec<Value>> {
    match context.get_val(array_id) {
        Some(&Value::Array(ref x)) => Ok(x.clone()),
        x => Err(Error::Render(format!("Tried to iterate over {:?}, which is not supported.", x))),
    }
}

fn token_as_int(arg: &Argument, context: &Context) -> Result<isize> {
    let value = match arg.evaluate(context)? {
        Value::Num(ref n) => *n,
        _ => return Error::renderer(&format!("{:?} is not a number.", arg)),
    };

    Ok(value as isize)
}

impl Renderable for For {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let mut range = match self.range {
            Range::Array(ref array_id) => try!(get_array(context, array_id)),

            Range::Counted(ref start_arg, ref stop_arg) => {
                let start = token_as_int(start_arg, context)?;
                let stop = token_as_int(stop_arg, context)?;
                (start..stop).map(|x| Value::Num(x as f32)).collect()
            }
        };

        let end = match self.limit {
            Some(n) => self.offset + n,
            None => range.len(),
        };

        let slice = if end > range.len() {
            &mut range[self.offset..]
        } else {
            &mut range[self.offset..end]
        };

        if self.reversed {
            slice.reverse();
        };

        match slice.len() {
            0 => {
                if let Some(ref t) = self.else_template {
                    t.render(context)
                } else {
                    Ok(None)
                }
            }

            range_len => {
                let mut ret = String::default();
                context.run_in_scope(|mut scope| {
                    let mut helper_vars: HashMap<String, Value> = HashMap::new();
                    helper_vars.insert("length".to_owned(), Value::Num(range_len as f32));

                    for (i, v) in slice.iter().enumerate() {
                        helper_vars.insert("index0".to_owned(), Value::Num(i as f32));
                        helper_vars.insert("index".to_owned(), Value::Num((i + 1) as f32));
                        helper_vars
                            .insert("rindex0".to_owned(), Value::Num((range_len - i - 1) as f32));
                        helper_vars.insert("rindex".to_owned(), Value::Num((range_len - i) as f32));
                        helper_vars.insert("first".to_owned(), Value::Bool(i == 0));
                        helper_vars.insert("last".to_owned(), Value::Bool(i == (range_len - 1)));

                        scope.set_local_val("forloop", Value::Object(helper_vars.clone()));
                        scope.set_local_val(&self.var_name, v.clone());
                        let inner = self.item_template
                            .render(&mut scope)?
                            .unwrap_or_else(String::new);
                        ret = ret + &inner;

                        // given that we're at the end of the loop body
                        // already, dealing with a `continue` signal is just
                        // clearing the interrupt and carrying on as normal. A
                        // `break` requires some special handling, though.
                        if let Some(Interrupt::Break) = scope.pop_interrupt() {
                            break;
                        }
                    }

                    Ok(Some(ret))
                })
            }
        }
    }
}


/// Extracts an attribute with an integer value from the token stream
fn int_attr(args: &mut Iter<Token>) -> Result<Option<usize>> {
    expect(args, &Token::Colon)?;
    match args.next() {
        Some(&Token::NumberLiteral(ref n)) => Ok(Some(*n as usize)),
        x => Error::parser("number", x),
    }
}

fn range_end_point(args: &mut Iter<Token>) -> Result<Token> {
    match args.next() {
        Some(id @ &Token::NumberLiteral(_)) |
        Some(id @ &Token::Identifier(_)) => Ok(id.clone()),
        x => Error::parser("number | Identifier", x),
    }
}

pub fn for_block(_tag_name: &str,
                 arguments: &[Token],
                 tokens: &[Element],
                 options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let var_name = match args.next() {
        Some(&Token::Identifier(ref x)) => x.clone(),
        x => return Error::parser("Identifier", x),
    };

    expect(&mut args, &Token::Identifier("in".to_owned()))?;

    let range = match args.next() {
        Some(&Token::Identifier(ref x)) => Range::Array(x.clone()),
        Some(&Token::OpenRound) => {
            // this might be a range, let's try and see
            let start = range_end_point(&mut args)?.to_arg()?;

            expect(&mut args, &Token::DotDot)?;

            let stop = range_end_point(&mut args)?.to_arg()?;

            expect(&mut args, &Token::CloseRound)?;

            Range::Counted(start, stop)
        }
        x => return Error::parser("Identifier or (", x),
    };

    // now we get to check for parameters...
    let mut limit: Option<usize> = None;
    let mut offset: usize = 0;
    let mut reversed = false;

    while let Some(token) = args.next() {
        match *token {
            Token::Identifier(ref attr) => {
                match attr.as_ref() {
                    "limit" => limit = int_attr(&mut args)?,
                    "offset" => offset = int_attr(&mut args)?.unwrap_or(0),
                    "reversed" => reversed = true,
                    _ => return Error::parser("limit | offset | reversed", Some(token)),
                }
            }
            _ => return Error::parser("Identifier", Some(token)),
        }
    }

    let (leading, trailing) = split_block(tokens, &["else"], options);
    let item_template = Template::new(parse(leading, options)?);

    let else_template = match trailing {
        Some(split) => {
            let parsed = parse(&split.trailing[1..], options)?;
            Some(Template::new(parsed))
        }
        None => None,
    };

    Ok(Box::new(For {
                    var_name: var_name,
                    range: range,
                    item_template: item_template,
                    else_template: else_template,
                    limit: limit,
                    offset: offset,
                    reversed: reversed,
                }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert("for".to_owned(),
                              (for_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn loop_over_array() {
        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Token::Identifier("name".to_owned()),
                                  Token::Identifier("in".to_owned()),
                                  Token::Identifier("array".to_owned())],
                                &compiler::tokenize("test {{name}} ").unwrap(),
                                &options)
            .unwrap();

        let mut data: Context = Default::default();
        data.set_val("array",
                     Value::Array(vec![Value::Num(22f32),
                                       Value::Num(23f32),
                                       Value::Num(24f32),
                                       Value::Str("wat".to_owned())]));
        let output = for_tag.render(&mut data).unwrap();
        assert_eq!(output, Some("test 22 test 23 test 24 test wat ".to_owned()));
    }


    #[test]
    fn loop_over_range_literals() {
        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Token::Identifier("name".to_owned()),
                                  Token::Identifier("in".to_owned()),
                                  Token::OpenRound,
                                  Token::NumberLiteral(42f32),
                                  Token::DotDot,
                                  Token::NumberLiteral(46f32),
                                  Token::CloseRound],
                                &compiler::tokenize("#{{forloop.index}} test {{name}} | ")
                                    .unwrap(),
                                &options)
            .unwrap();

        let mut data: Context = Default::default();
        let output = for_tag.render(&mut data).unwrap();
        assert_eq!(output,
                   Some("#1 test 42 | #2 test 43 | #3 test 44 | #4 test 45 | ".to_owned()));
    }

    #[test]
    fn loop_over_range_vars() {
        let text = concat!("{% for x in (alpha .. omega) %}",
                           "#{{forloop.index}} test {{x}}, ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_val("alpha", Value::Num(42f32));
        context.set_val("omega", Value::Num(46f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output,
                   Some("#1 test 42, #2 test 43, #3 test 44, #4 test 45, ".to_string()));
    }

    #[test]
    fn nested_forloops() {
        // test that nest nested for loops work, and that the
        // variable scopes between the inner and outer variable
        // scopes do not overlap.
        let text = concat!("{% for outer in (1..5) %}",
                           ">>{{forloop.index0}}:{{outer}}>>",
                           "{% for inner in (6..10) %}",
                           "{{outer}}:{{forloop.index0}}:{{inner}},",
                           "{% endfor %}",
                           ">>{{outer}}>>\n",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output,
                   Some(concat!(">>0:1>>1:0:6,1:1:7,1:2:8,1:3:9,>>1>>\n",
                                ">>1:2>>2:0:6,2:1:7,2:2:8,2:3:9,>>2>>\n",
                                ">>2:3>>3:0:6,3:1:7,3:2:8,3:3:9,>>3>>\n",
                                ">>3:4>>4:0:6,4:1:7,4:2:8,4:3:9,>>4>>\n")
                            .to_owned()));
    }

    #[test]
    fn nested_forloops_with_else() {
        // test that nested for loops parse their `else` blocks correctly
        let text = concat!("{% for x in (0..i) %}",
                           "{% for y in (0..j) %}inner{% else %}empty inner{% endfor %}",
                           "{% else %}",
                           "empty outer",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.set_val("i", Value::Num(0f32));
        context.set_val("j", Value::Num(0f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("empty outer".to_owned()));

        context.set_val("i", Value::Num(1f32));
        context.set_val("j", Value::Num(0f32));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("empty inner".to_owned()));
    }


    #[test]
    fn degenerate_range_is_safe() {
        // make sure that a degenerate range (i.e. where max < min)
        // doesn't result in an infinte loop
        let text = concat!("{% for x in (10 .. 0) %}", "{{x}}", "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("".to_string()));
    }

    #[test]
    fn limited_loop() {
        let text = concat!("{% for i in (1..100) limit:2 %}",
                           "{{ i }} ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("1 2 ".to_string()));
    }

    #[test]
    fn offset_loop() {
        let text = concat!("{% for i in (1..10) offset:4 %}",
                           "{{ i }} ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("5 6 7 8 9 ".to_string()));
    }

    #[test]
    fn offset_and_limited_loop() {
        let text = concat!("{% for i in (1..10) offset:4 limit:2 %}",
                           "{{ i }} ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("5 6 ".to_string()));
    }

    #[test]
    fn reversed_loop() {
        let text = concat!("{% for i in (1..10) reversed %}",
                           "{{ i }} ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("9 8 7 6 5 4 3 2 1 ".to_string()));
    }

    #[test]
    fn sliced_and_reversed_loop() {
        let text = concat!("{% for i in (1..10) reversed offset:1 limit:5%}",
                           "{{ i }} ",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("6 5 4 3 2 ".to_string()));
    }

    #[test]
    fn empty_loop_invokes_else_template() {
        let text = concat!("{% for i in (1..10) limit:0 %}",
                           "{{ i }} ",
                           "{% else %}",
                           "There are no items!",
                           "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("There are no items!".to_string()));
    }

    #[test]
    fn limit_greater_than_iterator_length() {
        let text = concat!("{% for i in (1..5) limit:10 %}", "{{ i }} ", "{% endfor %}");
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("1 2 3 4 ".to_string()));
    }

    #[test]
    fn loop_variables() {
        let for_tag = for_block("for",
                                &[Token::Identifier("v".to_owned()),
                                  Token::Identifier("in".to_owned()),
                                  Token::OpenRound,
                                  Token::NumberLiteral(100f32),
                                  Token::DotDot,
                                  Token::NumberLiteral(103f32),
                                  Token::CloseRound],
                                &compiler::tokenize(concat!("length: {{forloop.length}}, ",
                                                            "index: {{forloop.index}}, ",
                                                            "index0: {{forloop.index0}}, ",
                                                            "rindex: {{forloop.rindex}}, ",
                                                            "rindex0: {{forloop.rindex0}}, ",
                                                            "value: {{v}}, ",
                                                            "first: {{forloop.first}}, ",
                                                            "last: {{forloop.last}}\n"))
                                    .unwrap(),
                                &options())
            .unwrap();

        let mut data: Context = Default::default();
        let output = for_tag.render(&mut data).unwrap();
        assert_eq!(
            output,
            Some(
                concat!(
"length: 3, index: 1, index0: 0, rindex: 3, rindex0: 2, value: 100, first: true, last: false\n",
"length: 3, index: 2, index0: 1, rindex: 2, rindex0: 1, value: 101, first: false, last: false\n",
"length: 3, index: 3, index0: 2, rindex: 1, rindex0: 0, value: 102, first: false, last: true\n",
).to_owned(),
            )
        );
    }


    #[test]
    fn use_filters() {

        let for_tag = for_block("for",
                                &[Token::Identifier("name".to_owned()),
                                  Token::Identifier("in".to_owned()),
                                  Token::Identifier("array".to_owned())],
                                &compiler::tokenize("test {{name | shout}} ").unwrap(),
                                &options())
            .unwrap();

        let mut data: Context = Default::default();
        data.add_filter("shout",
                        ((|input, _args| if let &Value::Str(ref s) = input {
                              Ok(Value::Str(s.to_uppercase()))
                          } else {
                              interpreter::FilterError::invalid_type("Expected a string")
                          }) as interpreter::FnFilterValue)
                            .into());

        data.set_val("array",
                     Value::Array(vec![Value::str("alpha"),
                                       Value::str("beta"),
                                       Value::str("gamma")]));
        let output = for_tag.render(&mut data).unwrap();
        assert_eq!(output, Some("test ALPHA test BETA test GAMMA ".to_owned()));
    }
}
