use std::io::Write;

use liquid_error::Result;

use compiler::BlockElement;
use compiler::Language;
use compiler::TagBlock;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;

#[derive(Copy, Clone, Debug)]
struct Comment;

impl Renderable for Comment {
    fn render_to(&self, _writer: &mut Write, _context: &mut Context) -> Result<()> {
        Ok(())
    }
}

pub fn comment_block(
    tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &Language,
) -> Result<Box<Renderable>> {
    // no arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    while let Some(token) = tokens.next()? {
        // Only needs to parse tags. Expressions and raw text will never have side effects.
        if let BlockElement::Tag(tag) = token {
            if tag.name() == tag_name {
                // Parses `{% comment %}` tags (in order to allow nesting)
                tag.parse(&mut tokens, options)?;
            } else {
                // Other tags are parsed (because of possible side effects, such as in `{% raw %}`)
                // But their errors are ignored
                let _ = tag.parse(&mut tokens, options);
            }
        }
    }

    tokens.assert_empty();
    Ok(Box::new(Comment))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .blocks
            .register("comment", (comment_block as compiler::FnParseBlock).into());
        options
    }

    fn unit_parse(text: &str) -> String {
        let options = options();
        let template = compiler::parse(text, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();

        template.render(&mut context).unwrap()
    }

    #[test]
    fn test_comment() {
        let output = unit_parse("{% comment %} This is a test {% endcomment %}");
        assert_eq!(output, "");
    }
}
