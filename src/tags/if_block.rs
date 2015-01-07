use Renderable;
use Block;
use Value;
use Context;
use template::Template;
use LiquidOptions;
use tags::IfBlock;
use lexer::Token;
use lexer::Token::{Identifier, StringLiteral, NumberLiteral, Comparison};
use lexer::ComparisonOperator;
use lexer::ComparisonOperator::{
    Equals, NotEquals,
    LessThan, GreaterThan,
    LessThanEquals, GreaterThanEquals,
    Contains
};
use parser::parse;
use lexer::Element;
use lexer::Element::{Tag, Raw};
use std::default::Default;

struct If<'a>{
    lh : Token,
    comparison : ComparisonOperator,
    rh : Token,
    if_true: Template<'a>,
    if_false: Option<Template<'a>>
}

impl<'a> If<'a>{
    fn compare(&self, context: &Context) -> Result<bool, &'static str>{
        match (&self.lh, &self.rh)  {
            (&NumberLiteral(a), &NumberLiteral(b)) => Ok(compare_numbers(a, b, &self.comparison)),
            (&Identifier(ref var), &NumberLiteral(b)) => {
                match context.values.get(var.as_slice()) {
                    Some(&Value::Num(a)) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            },
            (&NumberLiteral(a), &Identifier(ref var)) => {
                match context.values.get(var.as_slice()) {
                    Some(&Value::Num(b)) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            }
            (&Identifier(ref var_a), &Identifier(ref var_b)) => {
                match (context.values.get(var_a.as_slice()), context.values.get(var_b.as_slice())) {
                    (Some(&Value::Num(a)), Some(&Value::Num(b))) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            }
            (_, _) => Err("not implemented yet!") // TODO
        }
    }
}

fn compare_numbers(a : f32, b : f32, comparison : &ComparisonOperator) -> bool{
    match comparison {
        &Equals => a == b,
        &NotEquals => a != b,
        &LessThan => a < b,
        &GreaterThan => a > b,
        &LessThanEquals => a <= b,
        &GreaterThanEquals => a >= b,
        &Contains => false, // TODO!!!
    }
}

impl<'a> Renderable for If<'a>{
    fn render(&self, context: &mut Context) -> Option<String>{
        if self.compare(context).unwrap_or(false){
            self.if_true.render(context)
        }else{
            match self.if_false {
                Some(ref template) => template.render(context),
                _ => None
            }
        }
    }
}

impl<'b> Block for IfBlock<'b>{
    fn initialize<'a>(&'a self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &'a LiquidOptions<'a>) -> Box<Renderable>{
        let mut args = arguments.iter();

        let lh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => panic!("Expected a value, found {}", x)
        };

        let comp = match args.next() {
            Some(&Comparison(ref x)) => x.clone(),
            x => panic!("Expected a comparison operator, found {}", x)
        };

        let rh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => panic!("Expected a value, found {}", x)
        };

        let else_block = vec![Identifier("else".to_string())];

        // advance until the end or an else token is reached
        // to gather everything to be executed if the condition is true
        let if_true_tokens = tokens.iter().take_while(|&x| match x  {
            &Tag(ref eb, _) => *eb != else_block,
            _ => true
        }).map(|x| x.clone()).collect();

        // gather everything after the else block
        // to be executed if the condition is false
        let if_false_tokens : Vec<Element> = tokens.iter().skip_while(|&x| match x  {
            &Tag(ref eb, _) => *eb != else_block,
            _ => true
        }).skip(1).map(|x| x.clone()).collect();

        let if_false = if if_false_tokens.len() > 0 {
            Some(Template::new(parse(if_false_tokens, options)))
        }else{
            None
        };

        box If{
            lh : lh,
            comparison : comp,
            rh : rh,
            if_true: Template::new(parse(if_true_tokens, options)),
            if_false: if_false
        } as Box<Renderable>
    }
}

#[test]
fn test_if() {
    let block = IfBlock;
    let options : LiquidOptions = Default::default();
    let if_tag = block.initialize("if", vec![NumberLiteral(5f32), Comparison(LessThan), NumberLiteral(6f32)][0..], vec![Raw("if true".to_string())], &options);
    assert_eq!(if_tag.render(&mut Default::default()).unwrap(), "if true".to_string());

    let else_tag = block.initialize("if", vec![NumberLiteral(7f32), Comparison(LessThan), NumberLiteral(6f32)][0..], vec![Raw("if true".to_string()), Tag(vec![Identifier("else".to_string())], "".to_string()), Raw("if false".to_string())], &options);
    assert_eq!(else_tag.render(&mut Default::default()).unwrap(), "if false".to_string());
}

