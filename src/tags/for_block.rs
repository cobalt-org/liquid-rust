use Renderable;
use context::Context;
use LiquidOptions;
use lexer::Token;
use lexer::Element;
use parser::parse;
use template::Template;
use lexer::Token::Identifier;
use value::Value;
use error::{Error, Result};

#[cfg(test)]
use std::default::Default;
#[cfg(test)]
use lexer::tokenize;

struct For<'a> {
    var_name: String,
    array_id: String,
    inner: Template<'a>,
}

fn get_array(context: &mut Context, array_id: &str) -> Result<Vec<Value>> {
    match context.get_val(array_id) {
        Some(&Value::Array(ref x)) => Ok(x.clone()),
        x => Err(Error::Render(format!("Tried to iterate over {:?}, which is not supported.", x))),
    }
}

impl<'a> Renderable for For<'a> {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        let arr = try!(get_array(context, &self.array_id));
        let mut ret = String::new();
        for i in arr {
            context.set_val(&self.var_name, i);
            ret = ret + &try!(self.inner.render(context)).unwrap_or("".to_owned());
        }
        Ok(Some(ret))
    }
}


pub fn for_block(_tag_name: &str,
                 arguments: &[Token],
                 tokens: Vec<Element>,
                 options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    let inner = try!(parse(&tokens, options));

    let var_name = match args.next() {
        Some(&Identifier(ref x)) => x.clone(),
        x => return Err(Error::Parser(format!("Expected an identifier, found {:?}", x))),
    };

    match args.next() {
        Some(&Identifier(ref x)) if x == "in" => (),
        x => return Err(Error::Parser(format!("Expected 'in', found {:?}", x))),
    }

    // TODO implement ranges
    let array_id = match args.next() {
        Some(&Identifier(ref x)) => x.clone(),
        x => return Err(Error::Parser(format!("Expected an identifier, found {:?}", x))),
    };

    Ok(Box::new(For {
        var_name: var_name,
        array_id: array_id,
        inner: Template::new(inner),
    }))
}

#[test]
fn test_for() {
    let options: LiquidOptions = Default::default();
    let for_tag = for_block("for",
                            &vec![Identifier("name".to_string()),
                                  Identifier("in".to_string()),
                                  Identifier("array".to_string())],
                            tokenize("test {{name}} ").unwrap(),
                            &options);

    let mut data: Context = Default::default();
    data.set_val("array",
                 Value::Array(vec![Value::Num(22f32),
                                   Value::Num(23f32),
                                   Value::Num(24f32),
                                   Value::Str("wat".to_string())]));
    assert_eq!(for_tag.unwrap().render(&mut data).unwrap(),
               Some("test 22 test 23 test 24 test wat ".to_string()));
}
