use Context;
use LiquidOptions;
use error::Result;

use syntax::Renderable;
use syntax::Token;
use syntax::Element;

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
    use super::*;

    #[test]
    fn test_comment() {
        let options: LiquidOptions = Default::default();
        let comment = comment_block("comment",
                                    &[],
                                    &vec![Element::Expression(vec![],
                                                              "This is a test".to_string())],
                                    &options);
        assert_eq!(comment.unwrap().render(&mut Default::default()).unwrap(),
                   None);
    }
}
