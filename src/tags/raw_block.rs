use Renderable;
use Value;
use Block;
use LiquidOptions;
use tags::RawBlock;
use lexer::Token;
use lexer::Element;
use lexer::Element::{Output, Tag, Raw};
use std::collections::HashMap;

struct RawT{
    content : String
}

impl Renderable for RawT{
    fn render(&self, context: &HashMap<String, Value>) -> String{
        self.content.to_string()
    }
}

impl Block for RawBlock{
    fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &LiquidOptions) -> Box<Renderable>{
        let content = tokens.iter().fold("".to_string(), |a, b|
                                         match b  {
                                            &Output(_, ref text) => text,
                                            &Tag(_, ref text) => text,
                                            &Raw(ref text) => text
                                         }.to_string() + a.to_string()
                                        );
        box RawT{content: content} as Box<Renderable>
    }
}

#[test]
fn test_raw() {
    let block = RawBlock;
    let options = LiquidOptions{blocks: HashMap::new(), tags: HashMap::new()};
    let raw = block.initialize("raw", vec![][0..], vec![Output(vec![], "This is a test".to_string())], &options);
    assert_eq!(raw.render(&HashMap::new()), "This is a test".to_string());
}
