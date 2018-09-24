use std::io::Write;

use liquid_error::{Result, ResultLiquidChainExt};

use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use interpreter::Context;
use interpreter::Renderable;

#[derive(Clone, Debug)]
struct RawT {
    content: String,
}

impl Renderable for RawT {
    fn render_to(&self, writer: &mut Write, _context: &mut Context) -> Result<()> {
        write!(writer, "{}", self.content).chain("Failed to render")?;
        Ok(())
    }
}

pub fn raw_block(
    _tag_name: &str,
    _arguments: &[Token],
    tokens: &[Element],
    _options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let content = tokens.iter().fold("".to_owned(), |a, b| {
        a + match *b {
            Element::Expression(_, ref text)
            | Element::Tag(_, ref text)
            | Element::Raw(ref text) => text,
        }
    });
    Ok(Box::new(RawT { content }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .blocks
            .insert("raw", (raw_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn raw_text() {
        let raw = raw_block(
            "raw",
            &[],
            &vec![Element::Expression(vec![], "This is a test".to_owned())],
            &options(),
        ).unwrap();
        let output = raw.render(&mut Default::default()).unwrap();
        assert_eq!(output, "This is a test");
    }

    #[test]
    fn raw_escaped() {
        let text = "{%raw%}{%if%}{%endraw%}";

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "{%if%}");
    }

    #[test]
    fn raw_mixed() {
        let text = "{%raw%}hello{%if%}world{%endraw%}";

        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "hello{%if%}world");
    }
}
