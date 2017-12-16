use error::Result;

use interpreter::Context;
use interpreter::Renderable;
use syntax::Element;
use syntax::LiquidOptions;
use syntax::Token;

#[derive(Clone, Debug)]
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

#[cfg(test)]
mod test {
    use super::*;
    use syntax;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .blocks
            .insert("raw".to_owned(), (raw_block as syntax::FnParseBlock).into());
        options
    }

    #[test]
    fn test_raw() {
        let raw = raw_block("raw",
                            &[],
                            &vec![Element::Expression(vec![], "This is a test".to_owned())],
                            &options())
            .unwrap();
        let output = raw.render(&mut Default::default()).unwrap();
        assert_eq!(output, Some("This is a test".to_owned()));
    }
}
