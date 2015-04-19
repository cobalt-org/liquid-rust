use Renderable;
use Block;
use Context;
use LiquidOptions;
use tags::CommentBlock;
use lexer::Token;
use lexer::Element;

#[cfg(test)]
use std::default::Default;
#[cfg(test)]
use lexer::Element::Expression;

struct CommentT;

impl Renderable for CommentT{
    fn render(&self, _context: &mut Context) -> Option<String>{
        None
    }
}

impl Block for CommentBlock{
    fn initialize(&self, _tag_name: &str, _arguments: &[Token], _tokens: Vec<Element>, _options : &LiquidOptions) -> Result<Box<Renderable>, String>{
        Ok(box CommentT as Box<Renderable>)
    }
}

#[test]
fn test_comment() {
    let block = CommentBlock;
    let options : LiquidOptions = Default::default();
    let comment = block.initialize("comment", &vec![], vec![Expression(vec![], "This is a test".to_string())], &options);
    assert_eq!(comment.unwrap().render(&mut Default::default()), None);
}
