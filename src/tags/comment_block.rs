use Renderable;
use context::Context;
use LiquidOptions;
use lexer::Token;
use lexer::Element;
use error::Result;

#[cfg(test)]
use std::default::Default;
#[cfg(test)]
use lexer::Element::Expression;

struct Comment;

impl Renderable for Comment {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        Ok(None)
    }
}

pub fn comment_block(_tag_name: &str,
                    _arguments: &[Token],
                    _tokens: Vec<Element>,
                    _options: &LiquidOptions)
                    -> Result<Box<Renderable>> {
    Ok(Box::new(Comment) as Box<Renderable>)
}

#[test]
fn test_comment() {
    let options: LiquidOptions = Default::default();
    let comment = comment_block("comment",
                                   &vec![],
                                   vec![Expression(vec![], "This is a test".to_string())],
                                   &options);
    assert_eq!(comment.unwrap().render(&mut Default::default()).unwrap(),
               None);
}
