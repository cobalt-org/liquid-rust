
use std::collections::HashMap;
use liquid::LiquidOptions;
use liquid::Tag;
use liquid::lexer::Token;
use liquid::Renderable;
use liquid::Context;
use liquid::value::Value;
use liquid::parse;
use std::default::Default;

#[test]
fn run() {
    struct Multiply{
        numbers: Vec<f32>
    }
    impl Renderable for Multiply{
        fn render(&self, context: &mut Context) -> Option<String>{
            let x = self.numbers.iter().fold(1f32, |a, &b| a * b);
            Some(x.to_string())
        }
    }

    struct MultiplyTag;
    impl Tag for MultiplyTag{
        fn initialize(&self, tag_name: &str, arguments: &[Token], options: &LiquidOptions) -> Box<Renderable>{
            let numbers = arguments.iter().filter_map( |x| {
                match x {
                    &Token::NumberLiteral(ref num) => Some(*num),
                    _ => None
                }
            }).collect();
            box Multiply{numbers: numbers} as Box<Renderable>
        }
    }

    let mut tags = HashMap::new();
    tags.insert("multiply".to_string(), box MultiplyTag as Box<Tag>);

    let mut options = LiquidOptions {
        blocks: Default::default(),
        tags: tags,
        error_mode: Default::default()
    };
    let template = parse("wat\n{{hello}}\n{{multiply 5 3}}{%raw%}{{multiply 5 3}}{%endraw%} test", &mut options).unwrap();

    let mut data : Context = Default::default();
    data.values.insert("hello".to_string(), Value::Str("world".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), "wat\nworld\n15{{multiply 5 3}} test".to_string());
}

