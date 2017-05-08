use Renderable;
use context::Context;
use LiquidOptions;
use token::Token;
use lexer::Element;
use error::Result;

struct Comment;

impl Renderable for Comment {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        Ok(None)
    }
}

pub fn comment_block(_tag_name: &str,
                     _arguments: &[Token],
                     _tokens: &[Element],
                     _options: &LiquidOptions)
                     -> Result<Box<Renderable>> {
    Ok(Box::new(Comment))
}

#[cfg(test)]
mod test {
    use LiquidOptions;
    use super::comment_block;
    use std::default::Default;
    use lexer::Element::Expression;

    #[test]
    fn test_comment() {
        let options: LiquidOptions = Default::default();
        let comment = comment_block("comment",
                                    &[],
                                    &vec![Expression(vec![], "This is a test".to_string())],
                                    &options);
        assert_eq!(comment
                       .unwrap()
                       .render(&mut Default::default())
                       .unwrap(),
                   None);
    }
}
