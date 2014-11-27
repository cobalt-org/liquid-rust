use Renderable;
use Block;
use tags::IfBlock;
use lexer::Token;
use std::collections::HashMap;

struct If;
impl Renderable for If{
    fn render(&self, context: &HashMap<String, String>) -> String{
        "".to_string()
    }
}

impl Block for IfBlock{
    fn initialize(&self, tag_name: &str, arguments: &[Token], tokens: String) -> Box<Renderable>{
        box If as Box<Renderable>
    }
}
