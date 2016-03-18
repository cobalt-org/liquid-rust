use Renderable;
use context::Context;
use LiquidOptions;
use lexer::Element;
use lexer::Token::{self, Identifier, OpenRound, CloseRound, NumberLiteral, DotDot, Colon};
use parser::parse;
use template::Template;
use value::Value;
use error::{Error, Result};
use lexer::Element::Tag;

use std::collections::HashMap;
use std::slice::Iter;

enum Range {
    Array (String),
    Counted (Token, Token)
}

struct For {
    var_name: String,
    range: Range,
    item_template: Template,
    else_template: Option<Template>,
    limit: Option<usize>,
    offset: usize,
    reversed: bool
}

fn get_array(context: &Context, array_id: &str) -> Result<Vec<Value>> {
    match context.get_val(array_id) {
        Some(&Value::Array(ref x)) => Ok(x.clone()),
        x => Err(Error::Render(format!("Tried to iterate over {:?}, which is not supported.", x))),
    }
}

fn get_number(context: &Context, id: &str) -> Result<f32> {
    match context.get_val(id) {
        Some(&Value::Num(ref x)) => Ok(*x),
        Some(ref x) => Err(Error::Render(format!("{:?} is not a number.", x))),
        None => Err(Error::Render(format!("No such variable {}.", id)))
    }
}

fn token_as_int(token: &Token, context: &Context) -> Result<isize> {
    let value = match token {
        &Identifier(ref id) => try!(get_number(context, id)),
        &NumberLiteral(ref n) => *n,
        _ => {
            let msg = format!("Expecting identifier or number, found {:?}", token);
            return Err(Error::Render(msg));
        }
    };
    Ok(value as isize)
}

impl Renderable for For {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let mut range = match self.range {
            Range::Array(ref array_id) => {
                try!(get_array(context, array_id))
            },

            Range::Counted(ref start_token, ref stop_token) => {
                let start = try!(token_as_int(start_token, context));
                let stop = try!(token_as_int(stop_token, context));
                (start..stop).map(|x| Value::Num(x as f32)).collect()
            }
        };

        let end = match self.limit {
            Some(n) => self.offset + n,
            None => range.len()
        };

        let slice = &mut range[self.offset .. end];
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
            },

            range_len => {
                let mut ret = String::default();
                context.run_in_scope(|mut scope| {
                    let mut helper_vars : HashMap<String, Value> = HashMap::new();
                    helper_vars.insert("length".to_owned(), Value::Num(range_len as f32));

                    for (i, v) in slice.iter().enumerate() {
                        helper_vars.insert("index0".to_owned(), Value::Num(i as f32));
                        helper_vars.insert("index".to_owned(), Value::Num((i + 1) as f32));
                        helper_vars.insert("rindex0".to_owned(), Value::Num((range_len - i - 1) as f32));
                        helper_vars.insert("rindex".to_owned(), Value::Num((range_len - i) as f32));
                        helper_vars.insert("first".to_owned(), Value::Bool(i == 0));
                        helper_vars.insert("last".to_owned(), Value::Bool(i == (range_len-1)));

                        scope.set_local_val("for_loop", Value::Object(helper_vars.clone()));
                        scope.set_local_val(&self.var_name, v.clone());
                        let inner = try!(self.item_template.render(&mut scope)).unwrap_or("".to_owned());
                        ret = ret + &inner;
                    }

                    Ok(Some(ret))
                })
            }
        }
    }
}

fn parser_err<T>(expected: &str, actual: Option<&Token>) -> Result<T> {
  Err(Error::Parser(format!("Expected {}, found {:?}", expected, actual)))
}

/// Extracts an attribute with an integer value from the token stream 
fn int_attr<'a>(args: &mut Iter<'a, Token>) -> Result<Option<usize>> {
    match args.next() {
        Some(&Colon) => (),
        x => return parser_err(":", x)
    };

    match args.next() {
        Some(&NumberLiteral(ref n)) => Ok(Some(*n as usize)),
        x => return parser_err("number", x)
    }
}

fn range_end_point<'a>(args: &mut Iter<'a, Token>) -> Result<Token> {
    let t = match args.next() {
        Some(id @ &NumberLiteral(_)) => id.clone(),
        Some(id @ &Identifier(_)) => id.clone(),
        x => return parser_err("number | Identifier", x)
    };
    Ok(t)
}

pub fn for_block(_tag_name: &str,
                 arguments: &[Token],
                 tokens: Vec<Element>,
                 options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let var_name = match args.next() {
        Some(&Identifier(ref x)) => x.clone(),
        x => return parser_err("Identifier", x)
    };

    match args.next() {
        Some(&Identifier(ref x)) if x == "in" => (),
        x => return parser_err("\'in\'", x),
    };

    let range = match args.next() {
        Some(&Identifier(ref x)) => Range::Array(x.clone()),
        Some(&OpenRound) => {
            // this might be a range, let's try and see
            let start = try!(range_end_point(&mut args));

            match args.next() {
                Some(&DotDot) => (),
                x => return parser_err("..", x)
            };

            let stop = try!(range_end_point(&mut args));

            match args.next() {
                Some(&CloseRound) => (),
                x => return parser_err(")", x)
            };

            Range::Counted (start, stop)
        },
        x => return parser_err("Identifier or (", x),
    };

    // now we get to check for parameters...
    let mut limit : Option<usize> = None;
    let mut offset : usize = 0;
    let mut reversed = false;

    while let Some(token) = args.next() {
        match token {
            &Identifier(ref attr) => {
                match attr.as_ref() {
                    "limit" => limit = try!(int_attr(&mut args)),
                    "offset" => offset = try!(int_attr(&mut args)).unwrap_or(0),
                    "reversed" => reversed = true,
                    _ => return parser_err("limit | offset | reversed", Some(token))
                }
            },
            _ => {
                return parser_err("Identifier", Some(token))
            }
        }
    }

    let else_tag = vec![Identifier("else".to_owned())];
    let is_not_else = |x : &&Element| {
        match *x {
            &Tag(ref tokens, _) => *tokens != else_tag,
            _ => true
        }
    };

    // finally, collect the templates for the item, and the optional "else"
    // block
    let item_tokens : Vec<Element> = tokens.iter()
                                           .take_while(&is_not_else)
                                           .cloned()
                                           .collect();
    let item_template = Template::new(try!(parse(&item_tokens, options)));

    let else_tokens : Vec<Element> = tokens.iter()
                                           .skip_while(&is_not_else)
                                           .skip(1)
                                           .cloned()
                                           .collect();
    let else_template = match &else_tokens {
        ts if ts.is_empty() => None,
        ts => Some(Template::new(try!(parse(ts, options))))
    };

    Ok(Box::new(For {
        var_name: var_name,
        range: range,
        item_template: item_template,
        else_template: else_template,
        limit: limit,
        offset: offset,
        reversed: reversed
    }))
}

#[cfg(test)]
mod test{
    use super::for_block;
    use parse;
    use LiquidOptions;
    use Renderable;
    use lexer::Token::{Identifier, OpenRound, CloseRound, NumberLiteral, DotDot};
    use lexer::tokenize;
    use value::Value;
    use std::default::Default;
    use context::Context;

    #[test]
    fn loop_over_array() {
        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Identifier("name".to_owned()),
                                  Identifier("in".to_owned()),
                                  Identifier("array".to_owned())],
                                tokenize("test {{name}} ").unwrap(),
                                &options);

        let mut data: Context = Default::default();
        data.set_val("array",
                     Value::Array(vec![Value::Num(22f32),
                                       Value::Num(23f32),
                                       Value::Num(24f32),
                                       Value::Str("wat".to_owned())]));
        assert_eq!(for_tag.unwrap().render(&mut data).unwrap(),
                   Some("test 22 test 23 test 24 test wat ".to_owned()));
    }


    #[test]
    fn loop_over_range_literals() {
        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Identifier("name".to_owned()),
                                  Identifier("in".to_owned()),
                                  OpenRound,
                                  NumberLiteral(42f32),
                                  DotDot,
                                  NumberLiteral(46f32),
                                  CloseRound],
                                tokenize("#{{for_loop.index}} test {{name}} | ").unwrap(),
                                &options);

        let mut data: Context = Default::default();
        assert_eq!(for_tag.unwrap().render(&mut data).unwrap(),
                   Some("#1 test 42 | #2 test 43 | #3 test 44 | #4 test 45 | ".to_owned()));
    }

    #[test]
    fn loop_over_range_vars() {
        let text = concat!(
            "{% for x in (alpha .. omega) %}",
            "#{{for_loop.index}} test {{x}}, ",
            "{% endfor %}"
        );

        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        context.set_val("alpha", Value::Num(42f32));
        context.set_val("omega", Value::Num(46f32));
        let output = template.render(&mut context);
        assert_eq!(
            output.unwrap(),
            Some("#1 test 42, #2 test 43, #3 test 44, #4 test 45, ".to_string()));
    }

    #[test]
    fn degenerate_range_is_safe() {
        // make sure that a degenerate range (i.e. where max < min)
        // doesn't result in an infinte loop
        let text = concat!(
            "{% for x in (10 .. 0) %}",
            "{{x}}",
            "{% endfor %}"
        );
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("".to_string()));
    }

    #[test]
    fn limited_loop() {
        let text = concat!(
            "{% for i in (1..100) limit:2 %}",
            "{{ i }} ",
            "{% endfor %}");
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("1 2 ".to_string()));
    }

    #[test]
    fn offset_loop() {
        let text = concat!(
            "{% for i in (1..10) offset:4 %}",
            "{{ i }} ",
            "{% endfor %}");
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("5 6 7 8 9 ".to_string()));
    }

    #[test]
    fn offset_and_limited_loop() {
        let text = concat!(
            "{% for i in (1..10) offset:4 limit:2 %}",
            "{{ i }} ",
            "{% endfor %}");
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("5 6 ".to_string()));
    }

    #[test]
    fn reversed_loop() {
        let text = concat!(
            "{% for i in (1..10) reversed %}",
            "{{ i }} ",
            "{% endfor %}");
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("9 8 7 6 5 4 3 2 1 ".to_string()));
    }

    #[test]
    fn sliced_and_reversed_loop() {
        let text = concat!(
            "{% for i in (1..10) reversed offset:1 limit:5%}",
            "{{ i }} ",
            "{% endfor %}");
        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("6 5 4 3 2 ".to_string()));
    }

    #[test]
    fn empty_loop_invokes_else_template() {
        let text = concat!(
            "{% for i in (1..10) limit:0 %}",
            "{{ i }} ",
            "{% else %}",
            "There are no items!",
            "{% endfor %}");

        let template = parse(text, Default::default()).unwrap();
        let mut context = Context::new();
        let output = template.render(&mut context);
        assert_eq!(output.unwrap(), Some("There are no items!".to_string()));
    }

    #[test]
    fn loop_variables() {
        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Identifier("v".to_owned()),
                                  Identifier("in".to_owned()),
                                  OpenRound,
                                  NumberLiteral(100f32),
                                  DotDot,
                                  NumberLiteral(103f32),
                                  CloseRound],
                                tokenize(concat!(
                                         "length: {{for_loop.length}}, ",
                                         "index: {{for_loop.index}}, ",
                                         "index0: {{for_loop.index0}}, ",
                                         "rindex: {{for_loop.rindex}}, ",
                                         "rindex0: {{for_loop.rindex0}}, ",
                                         "value: {{v}}, ",
                                         "first: {{for_loop.first}}, ",
                                         "last: {{for_loop.last}}\n")).unwrap(),
                                &options);

        let mut data: Context = Default::default();
        assert_eq!(for_tag.unwrap().render(&mut data).unwrap(),
                   Some(concat!(
                    "length: 3, index: 1, index0: 0, rindex: 3, rindex0: 2, value: 100, first: true, last: false\n",
                    "length: 3, index: 2, index0: 1, rindex: 2, rindex0: 1, value: 101, first: false, last: false\n",
                    "length: 3, index: 3, index0: 2, rindex: 1, rindex0: 0, value: 102, first: false, last: true\n",
                    ).to_owned()));
    }


    #[test]
    fn use_filters() {
        use filters::FilterError;

        let options: LiquidOptions = Default::default();
        let for_tag = for_block("for",
                                &[Identifier("name".to_owned()),
                                  Identifier("in".to_owned()),
                                  Identifier("array".to_owned())],
                                tokenize("test {{name | shout}} ").unwrap(),
                                &options);

        let mut data: Context = Default::default();
        data.add_filter("shout", Box::new(|input, _args| {
            if let &Value::Str(ref s) = input {
                Ok(Value::Str(s.to_uppercase()))
            } else {
                FilterError::invalid_type("Expected a string")
            }
        }));

        data.set_val("array",
                     Value::Array(vec![Value::str("alpha"),
                                       Value::str("beta"),
                                       Value::str("gamma")
                                       ]));
        assert_eq!(for_tag.unwrap().render(&mut data).unwrap(),
                   Some("test ALPHA test BETA test GAMMA ".to_owned()));
    }
}
