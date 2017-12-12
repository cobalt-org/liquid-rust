use LiquidOptions;
use context::Context;
use error::Result;

use syntax::Element;
use syntax::Renderable;
use syntax::Token;

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
            Element::Expression(_, ref text) |
            Element::Tag(_, ref text) |
            Element::Raw(ref text) => text,
        }.to_owned() + &a
    });
    Ok(Box::new(RawT { content: content }))
}

#[test]
fn test_raw() {
    use super::*;

    let options: LiquidOptions = Default::default();
    let raw = raw_block("raw",
                        &[],
                        &vec![Element::Expression(vec![], "This is a test".to_owned())],
                        &options);
    assert_eq!(raw.unwrap().render(&mut Default::default()).unwrap(),
               Some("This is a test".to_owned()));
}
