use Renderable;
use Block;
use context::Context;
use LiquidOptions;
use tags::RawBlock;
use lexer::Token;
use lexer::Element;
use lexer::Element::{Expression, Tag, Raw};

#[cfg(test)]
use std::default::Default;

struct RawT{
    content : String
}

impl Renderable for RawT{
    fn render(&self, _context: &mut Context) -> Option<String>{
        Some(self.content.to_string())
    }
}

impl Block for RawBlock{
    fn initialize(&self, _tag_name: &str, _arguments: &[Token], tokens: Vec<Element>, _options : &LiquidOptions) -> Result<Box<Renderable>, String>{
        let content = tokens.iter().fold("".to_string(), |a, b|
                                         match b  {
                                            &Expression(_, ref text) => text,
                                            &Tag(_, ref text) => text,
                                            &Raw(ref text) => text
                                         }.to_string() + &a
                                        );
        Ok(Box::new(RawT{content: content}) as Box<Renderable>)
    }
}

#[test]
fn test_raw() {
    let block = RawBlock;
    let options : LiquidOptions = Default::default();
    let raw = block.initialize("raw", &vec![], vec![Expression(vec![], "This is a test".to_string())], &options);
    assert_eq!(raw.unwrap().render(&mut Default::default()).unwrap(), "This is a test".to_string());
}
