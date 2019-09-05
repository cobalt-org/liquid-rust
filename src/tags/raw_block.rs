use std::io::Write;

use liquid_error::{Result, ResultLiquidReplaceExt};

use compiler::BlockReflection;
use compiler::Language;
use compiler::ParseBlock;
use compiler::TagBlock;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;

#[derive(Clone, Debug)]
struct RawT {
    content: String,
}

impl Renderable for RawT {
    fn render_to(&self, writer: &mut dyn Write, _context: &mut Context) -> Result<()> {
        write!(writer, "{}", self.content).replace("Failed to render")?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RawBlock;

impl RawBlock {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BlockReflection for RawBlock {
    fn start_tag(&self) -> &'static str {
        "raw"
    }

    fn end_tag(&self) -> &'static str {
        "endraw"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseBlock for RawBlock {
    fn parse(
        &self,
        mut arguments: TagTokenIter,
        mut tokens: TagBlock,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        // no arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        let content = tokens.escape_liquid(false)?.to_string();

        tokens.assert_empty();
        Ok(Box::new(RawT { content }))
    }

    fn reflection(&self) -> &dyn BlockReflection {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;

    fn options() -> Language {
        let mut options = Language::default();
        options.blocks.register("raw", RawBlock.into());
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
    fn raw_text() {
        let output = unit_parse("{%raw%}This is a test{%endraw%}");
        assert_eq!(output, "This is a test");
    }

    #[test]
    fn raw_escaped() {
        let output = unit_parse("{%raw%}{%if%}{%endraw%}");
        assert_eq!(output, "{%if%}");
    }

    #[test]
    fn raw_mixed() {
        let output = unit_parse("{%raw%}hello{%if%}world{%endraw%}");
        assert_eq!(output, "hello{%if%}world");
    }
}
