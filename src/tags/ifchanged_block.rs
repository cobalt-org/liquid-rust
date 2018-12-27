use std::io::Write;

use liquid_error::{Result, ResultLiquidExt, ResultLiquidReplaceExt};

use compiler::Language;
use compiler::TagBlock;
use compiler::TagTokenIter;
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
            .trace_with(|| self.trace().into())?;

        let rendered = String::from_utf8(rendered).expect("render only writes UTF-8");
        if context.get_register_mut::<State>().has_changed(&rendered) {
            write!(writer, "{}", rendered).replace("Failed to render")?;
        }

        Ok(())
    }
}

pub fn ifchanged_block(
    _tag_name: &str,
    mut arguments: TagTokenIter,
    mut tokens: TagBlock,
    options: &Language,
) -> Result<Box<Renderable>> {
    // no arguments should be supplied, trying to supply them is an error
    arguments.expect_nothing()?;

    let if_changed = Template::new(tokens.parse_all(options)?);

    tokens.assert_empty();
    Ok(Box::new(IfChanged { if_changed }))
}

/// Remembers the content of the last rendered `ifstate` block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct State {
    last_rendered: Option<String>,
}

impl State {
    /// Checks whether or not a new rendered `&str` is different from
    /// `last_rendered` and updates `last_rendered` value to the new value.
    fn has_changed(&mut self, rendered: &str) -> bool {
        let has_changed = if let Some(last_rendered) = &self.last_rendered {
            last_rendered != rendered
        } else {
            true
        };
        self.last_rendered = Some(rendered.to_owned());

        has_changed
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;

    fn options() -> Language {
        let mut options = Language::default();
        options.blocks.register(
            "ifchanged",
            (ifchanged_block as compiler::FnParseBlock).into(),
        );
        options
            .blocks
            .register("for", (tags::for_block as compiler::FnParseBlock).into());
        options
            .blocks
            .register("if", (tags::if_block as compiler::FnParseBlock).into());
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
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "\nHey! \nHey! Numbers are now bigger than 5!");
    }
}
