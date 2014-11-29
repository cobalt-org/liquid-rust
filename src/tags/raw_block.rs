use Renderable;
use Block;
use LiquidOptions;
use tags::RawBlock;
use lexer::Token;
use parser::parse;
use lexer::Element;
use std::collections::HashMap;

struct Raw{
    content : String
}

impl Renderable for Raw{
    fn render(&self, context: &HashMap<String, String>) -> String{
        self.content.to_string()
    }
}

impl Block for RawBlock{
    fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &LiquidOptions) -> Box<Renderable>{
        println!("init");
        let content = tokens.iter().fold("".to_string(), |a, b| a + b.to_string());
        box Raw{content: content} as Box<Renderable>
    }
}
