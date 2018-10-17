use std::io::Write;

use liquid_error::{Result, ResultLiquidChainExt, ResultLiquidExt};

use compiler::parse;
use compiler::Element;
use compiler::LiquidOptions;
use compiler::Token;
use interpreter::Context;
use interpreter::Renderable;
use interpreter::Template;

#[derive(Debug)]
struct IfChanged {
    if_changed: Template,
}

impl IfChanged {
    fn trace(&self) -> String {
        "{{% ifchanged %}}".to_owned()
    }
}

impl Renderable for IfChanged {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let mut rendered = Vec::new();
        self.if_changed
            .render_to(&mut rendered, context)
            .trace_with(|| self.trace())?;

        let rendered = String::from_utf8(rendered).expect("render only writes UTF-8");
        if context.ifchanged().has_changed(&rendered) {
            write!(writer, "{}", rendered).chain("Failed to render")?;
        }

        Ok(())
    }
}

pub fn ifchanged_block(
    _tag_name: &str,
    _arguments: &[Token],
    tokens: &[Element],
    options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let if_changed = Template::new(parse(&tokens[..], options)?);
    Ok(Box::new(IfChanged { if_changed }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.blocks.insert(
            "ifchanged",
            (ifchanged_block as compiler::FnParseBlock).into(),
        );
        options
            .blocks
            .insert("for", (tags::for_block as compiler::FnParseBlock).into());
        options
            .blocks
            .insert("if", (tags::if_block as compiler::FnParseBlock).into());
        options
    }

    #[test]
    fn test_ifchanged_block() {
        let text = concat!(
            "{% for a in (0..10) %}",
            "{% ifchanged %}",
            "\nHey! ",
            "{% if a > 5 %}",
            "Numbers are now bigger than 5!",
            "{% endif %}",
            "{% endifchanged %}",
            "{% endfor %}",
        );
        let tokens = compiler::tokenize(&text).unwrap();
        let template = compiler::parse(&tokens, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "\nHey! \nHey! Numbers are now bigger than 5!");
    }
}
