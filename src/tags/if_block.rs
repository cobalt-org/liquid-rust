use Renderable;
use Block;
use value::Value;
use context::Context;
use template::Template;
use LiquidOptions;
use tags::IfBlock;
use lexer::Token;
use lexer::Token::{Identifier, StringLiteral, NumberLiteral, Comparison};
use lexer::ComparisonOperator;
use lexer::ComparisonOperator::{Equals, NotEquals, LessThan, GreaterThan, LessThanEquals,
                                GreaterThanEquals, Contains};
use parser::parse;
use lexer::Element;
use lexer::Element::Tag;
use error::{Error, Result};

struct If<'a> {
    lh: Token,
    comparison: ComparisonOperator,
    rh: Token,
    if_true: Template<'a>,
    if_false: Option<Template<'a>>,
}

fn token_to_val(token: &Token, context: &Context) -> Option<Value> {
    match token {
        &StringLiteral(ref x) => Some(Value::Str(x.to_string())),
        &NumberLiteral(x) => Some(Value::Num(x)),
        &Identifier(ref x) => match context.get_val(x) {
            Some(y) => Some(y.clone()),
            None => None,
        },
        _ => None,
    }
}

impl<'a> If<'a>{
    fn compare(&self, context: &Context) -> bool {
        let a = token_to_val(&self.lh, context);
        if let None = a {
            return false;
        }
        let b = token_to_val(&self.rh, context);
        if let None = b {
            return false;
        }
        match &self.comparison {
            &Equals => a == b,
            &NotEquals => a != b,
            &LessThan => a < b,
            &GreaterThan => a > b,
            &LessThanEquals => a <= b,
            &GreaterThanEquals => a >= b,
            &Contains => false, // TODO!!!
        }
    }
}

impl<'a> Renderable for If<'a>{
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

impl Block for IfBlock{
    fn initialize<'a>(&'a self,
                      _tag_name: &str,
                      arguments: &[Token],
                      tokens: Vec<Element>,
                      options: &'a LiquidOptions)
                      -> Result<Box<Renderable + 'a>> {
        let mut args = arguments.iter();

        let lh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => return Err(Error::Parser(format!("Expected a value, found {:?}", x))),
        };

        let comp = match args.next() {
            Some(&Comparison(ref x)) => x.clone(),
            x =>
                return Err(Error::Parser(format!("Expected a comparison operator, found {:?}", x))),
        };

        let rh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => return Err(Error::Parser(format!("Expected a value, found {:?}", x))),
        };

        let else_block = vec![Identifier("else".to_string())];

        // advance until the end or an else token is reached
        // to gather everything to be executed if the condition is true
        let if_true_tokens: Vec<Element> = tokens.iter()
                                                 .take_while(|&x| {
                                                     match x {
                                                         &Tag(ref eb, _) => *eb != else_block,
                                                         _ => true,
                                                     }
                                                 })
                                                 .map(|x| x.clone())
                                                 .collect();

        // gather everything after the else block
        // to be executed if the condition is false
        let if_false_tokens: Vec<Element> = tokens.iter()
                                                  .skip_while(|&x| {
                                                      match x {
                                                          &Tag(ref eb, _) => *eb != else_block,
                                                          _ => true,
                                                      }
                                                  })
                                                  .skip(1)
                                                  .map(|x| x.clone())
                                                  .collect();

        // if false is None if there is no block to execute
        let if_false = if if_false_tokens.len() > 0 {
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
        }) as Box<Renderable>)
    }
}

#[cfg(test)]
mod test{
    use LiquidOptions;
    use Block;
    use std::default::Default;
    use tags::IfBlock;
    use lexer::Element::Raw;
    use lexer::Token::{Identifier, StringLiteral, NumberLiteral, Comparison};
    use lexer::Element::Tag;
    use lexer::ComparisonOperator::{LessThan, Equals};

    #[test]
    fn test_number_comparison() {
        let block = IfBlock;
        let options: LiquidOptions = Default::default();
        // 5 < 6 then "if true" else "if false"
        let if_tag = block.initialize("if",
                                      &vec![NumberLiteral(5f32),
                                            Comparison(LessThan),
                                            NumberLiteral(6f32)],
                                      vec![Raw("if true".to_string())],
                                      &options);
        assert_eq!(if_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if true".to_string()));

        // 7 < 6 then "if true" else "if false"
        let else_tag = block.initialize("if",
                                        &vec![NumberLiteral(7f32),
                                              Comparison(LessThan),
                                              NumberLiteral(6f32)],
                                        vec![Raw("if true".to_string()),
                                             Tag(vec![Identifier("else".to_string())],
                                                 "".to_string()),
                                             Raw("if false".to_string())],
                                        &options);
        assert_eq!(else_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if false".to_string()));
    }

    #[test]
    fn test_string_comparison() {
        let block = IfBlock;
        let options: LiquidOptions = Default::default();
        // "one" == "one" then "if true" else "if false"
        let if_tag = block.initialize("if",
                                      &vec![StringLiteral("one".to_string()),
                                            Comparison(Equals),
                                            StringLiteral("one".to_string())],
                                      vec![Raw("if true".to_string())],
                                      &options);
        assert_eq!(if_tag.unwrap().render(&mut Default::default()).unwrap(),
                   Some("if true".to_string()));

        // "one" == "two"
        let else_tag = block.initialize("if",
                                        &vec![StringLiteral("one".to_string()),
                                              Comparison(Equals),
                                              StringLiteral("two".to_string())],
                                        vec![Raw("if true".to_string())],
                                        &options);
        assert_eq!(else_tag.unwrap().render(&mut Default::default()).unwrap(), None);
    }
}
