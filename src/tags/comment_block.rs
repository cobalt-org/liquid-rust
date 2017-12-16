use error::Result;

use interpreter::Context;
use interpreter::Renderable;
use syntax::Element;
use syntax::LiquidOptions;
use syntax::Token;

#[derive(Copy, Clone, Debug)]
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
    use syntax;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert("comment".to_owned(),
                              (comment_block as syntax::FnParseBlock).into());
        options
    }

    #[test]
    fn test_comment() {
        let options = options();
        let comment = comment_block("comment",
                                    &[],
                                    &vec![Element::Expression(vec![],
                                                              "This is a test".to_string())],
                                    &options);
        assert_eq!(comment.unwrap().render(&mut Default::default()).unwrap(),
                   None);
    }
}
