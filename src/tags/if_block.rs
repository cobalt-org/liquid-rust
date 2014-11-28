use Renderable;
use Block;
use LiquidOptions;
use tags::IfBlock;
use lexer::Token;
use parser::parse;
use lexer::Element;
use std::collections::HashMap;

struct If;
impl Renderable for If{
    fn render(&self, context: &HashMap<String, String>) -> String{
        "".to_string()
    }
}

impl Block for IfBlock{
    fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &LiquidOptions) -> Box<Renderable>{
        let test = parse(tokens, options);
        box If as Box<Renderable>
    }
}
