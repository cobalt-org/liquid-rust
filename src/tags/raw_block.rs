use Renderable;
use context::Context;
use LiquidOptions;
use token::Token;
use lexer::Element::{self, Expression, Tag, Raw};
use error::Result;

struct RawT {
    content: String,
}

impl Renderable for RawT {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        Ok(Some(self.content.to_owned()))
    }
}

pub fn raw_block(_tag_name: &str,
                 _arguments: &[Token],
                 tokens: &[Element],
                 _options: &LiquidOptions)
                 -> Result<Box<Renderable>> {
    let content = tokens.iter().fold("".to_owned(), |a, b| {
        match *b {
                Expression(_, ref text) |
                Tag(_, ref text) |
                Raw(ref text) => text,
            }
            .to_owned() + &a
    });
    Ok(Box::new(RawT { content: content }))
}

#[test]
fn test_raw() {
    use std::default::Default;

    let options: LiquidOptions = Default::default();
    let raw = raw_block("raw",
                        &[],
                        &vec![Expression(vec![], "This is a test".to_owned())],
                        &options);
    assert_eq!(raw.unwrap().render(&mut Default::default()).unwrap(),
               Some("This is a test".to_owned()));
}
