extern crate liquid;

use std::collections::HashMap;
use liquid::LiquidOptions;
use liquid::Tag;
use liquid::lexer::Token;
use liquid::Renderable;
use liquid::Context;
use liquid::Value;
use liquid::parse;
use liquid::Error;
use std::default::Default;

#[test]
fn run() {
    struct Multiply{
        numbers: Vec<f32>
    }
    impl Renderable for Multiply{
        fn render(&self, _context: &mut Context) -> Result<Option<String>, Error> {
            let x = self.numbers.iter().fold(1f32, |a, &b| a * b);
            Ok(Some(x.to_string()))
        }
    }

    fn multiply_tag(_tag_name: &str, arguments: &[Token], _options: &LiquidOptions) -> Box<Renderable>{
        let numbers = arguments.iter().filter_map( |x| {
            match x {
                &Token::NumberLiteral(ref num) => Some(*num),
                _ => None
            }
        }).collect();
        Box::new(Multiply{numbers: numbers}) as Box<Renderable>
    }

    let mut tags = HashMap::new();
    tags.insert("multiply".to_string(), Box::new(multiply_tag) as Box<Tag>);

    let options = LiquidOptions {
        blocks: Default::default(),
        tags: tags,
        error_mode: Default::default()
    };
    let template = parse("wat\n{{hello}}\n{{multiply 5 3}}{%raw%}{{multiply 5 3}}{%endraw%} test", options).unwrap();

    let mut data = Context::new() ;
    data.set_val("hello", Value::Str("world".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("wat\nworld\n15{{multiply 5 3}} test".to_string()));
}
