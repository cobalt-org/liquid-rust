use Renderable;
use value::Value;
use context::Context;
use template::Template;
use LiquidOptions;
use lexer::Token::{self, Identifier, StringLiteral, NumberLiteral, Comparison};
use lexer::ComparisonOperator::{self, Equals, NotEquals, LessThan, GreaterThan, LessThanEquals,
                                GreaterThanEquals, Contains};
use parser::parse;
use lexer::Element::{self, Tag};
use error::{Error, Result};

struct If {
    lh: Token,
    comparison: ComparisonOperator,
    rh: Token,
    if_true: Template,
    if_false: Option<Template>,
}

fn token_to_val(token: &Token, context: &Context) -> Option<Value> {
    match *token {
        StringLiteral(ref x) => Some(Value::Str(x.to_owned())),
        NumberLiteral(x) => Some(Value::Num(x)),
        Identifier(ref x) => {
            match context.get_val(x) {
                Some(y) => Some(y.clone()),
                None => None,
            }
        }
        _ => None,
    }
}

impl If {
    fn compare(&self, context: &Context) -> bool {
        let a = token_to_val(&self.lh, context);
        if let None = a {
            return false;
        }
        let b = token_to_val(&self.rh, context);
        if let None = b {
            return false;
        }
        match self.comparison {
            Equals => a == b,
            NotEquals => a != b,
            LessThan => a < b,
            GreaterThan => a > b,
            LessThanEquals => a <= b,
            GreaterThanEquals => a >= b,
            Contains => false, // TODO!!!
        }
    }
}

impl Renderable for If {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {
        if self.compare(context) {
            self.if_true.render(context)
        } else {
            match self.if_false {
                Some(ref template) => template.render(context),
                _ => Ok(None),
            }
        }
    }
}

pub fn if_block(_tag_name: &str,
                arguments: &[Token],
                tokens: Vec<Element>,
                options: &LiquidOptions)
                -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    let lh = match args.next() {
        Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
        Some(&NumberLiteral(x)) => NumberLiteral(x),
        Some(&Identifier(ref x)) => Identifier(x.clone()),
        x => return Err(Error::Parser(format!("Expected a value, found {:?}", x))),
    };

    let comp = match args.next() {
        Some(&Comparison(ref x)) => x.clone(),
        x => return Err(Error::Parser(format!("Expected a comparison operator, found {:?}", x))),
    };

    let rh = match args.next() {
        Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
        Some(&NumberLiteral(x)) => NumberLiteral(x),
        Some(&Identifier(ref x)) => Identifier(x.clone()),
        x => return Err(Error::Parser(format!("Expected a value, found {:?}", x))),
    };

    let else_block = vec![Identifier("else".to_owned())];

    // advance until the end or an else token is reached
    // to gather everything to be executed if the condition is true
    let if_true_tokens: Vec<Element> = tokens.iter()
                                             .take_while(|&x| {
                                                 match *x {
                                                     Tag(ref eb, _) => *eb != else_block,
                                                     _ => true,
                                                 }
                                             })
                                             .cloned()
                                             .collect();

    // gather everything after the else block
    // to be executed if the condition is false
    let if_false_tokens: Vec<Element> = tokens.iter()
                                              .skip_while(|&x| {
                                                  match *x {
                                                      Tag(ref eb, _) => *eb != else_block,
                                                      _ => true,
                                                  }
                                              })
                                              .skip(1)
                                              .cloned()
                                              .collect();

    // if false is None if there is no block to execute
    let if_false = if !if_false_tokens.is_empty() {
        Some(Template::new(try!(parse(&if_false_tokens, options))))
    } else {
        None
    };

    let if_true = Template::new(try!(parse(&if_true_tokens, options)));

    Ok(Box::new(If {
        lh: lh,
        comparison: comp,
        rh: rh,
        if_true: if_true,
        if_false: if_false,
    }))
}

#[cfg(test)]
mod test {
    use LiquidOptions;
    use std::default::Default;
    use tags::if_block;
    use lexer::Element::{Raw, Tag};
    use lexer::Token::{Identifier, StringLiteral, NumberLiteral, Comparison};
    use lexer::ComparisonOperator::{LessThan, Equals};

    #[test]
    fn test_number_comparison() {
        let options: LiquidOptions = Default::default();
        // 5 < 6 then "if true" else "if false"
        let if_tag = if_block("if",
                              &[NumberLiteral(5f32), Comparison(LessThan), NumberLiteral(6f32)],
                              vec![Raw("if true".to_owned())],
                              &options);
        assert_eq!(if_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if true".to_owned()));

        // 7 < 6 then "if true" else "if false"
        let else_tag = if_block("if",
                                &[NumberLiteral(7f32), Comparison(LessThan), NumberLiteral(6f32)],
                                vec![Raw("if true".to_owned()),
                                     Tag(vec![Identifier("else".to_owned())], "".to_owned()),
                                     Raw("if false".to_owned())],
                                &options);
        assert_eq!(else_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if false".to_owned()));
    }

    #[test]
    fn test_string_comparison() {
        let options: LiquidOptions = Default::default();
        // "one" == "one" then "if true" else "if false"
        let if_tag = if_block("if",
                              &[StringLiteral("one".to_owned()),
                                Comparison(Equals),
                                StringLiteral("one".to_owned())],
                              vec![Raw("if true".to_owned())],
                              &options);
        assert_eq!(if_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if true".to_owned()));

        // "one" == "two"
        let else_tag = if_block("if",
                                &[StringLiteral("one".to_owned()),
                                  Comparison(Equals),
                                  StringLiteral("two".to_owned())],
                                vec![Raw("if true".to_owned())],
                                &options);
        assert_eq!(else_tag.unwrap().render(&mut Default::default()).unwrap(),
                   None);
    }
}
