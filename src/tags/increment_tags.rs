use std::io::Write;

use liquid_error::{Result, ResultLiquidChainExt};

use compiler::unexpected_token_error;
use compiler::LiquidOptions;
use compiler::Token;
use interpreter::Context;
use interpreter::Renderable;
use value::Value;

#[derive(Clone, Debug)]
struct Increment {
    id: String,
}

impl Renderable for Increment {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let mut val = context
            .stack()
            .get_index(&self.id)
            .and_then(|i| i.as_scalar())
            .and_then(|i| i.to_integer())
            .unwrap_or(0);

        write!(writer, "{}", val).chain("Failed to render")?;
        val += 1;
        context
            .stack_mut()
            .set_index(self.id.to_owned(), Value::scalar(val));
        Ok(())
    }
}

pub fn increment_tag(
    _tag_name: &str,
    arguments: &[Token],
    _options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let id = match args.next() {
        Some(&Token::Identifier(ref id)) => id.clone(),
        x => return Err(unexpected_token_error("identifier", x)),
    };

    Ok(Box::new(Increment { id }))
}

#[derive(Clone, Debug)]
struct Decrement {
    id: String,
}

impl Renderable for Decrement {
    fn render_to(&self, writer: &mut Write, context: &mut Context) -> Result<()> {
        let mut val = context
            .stack()
            .get_index(&self.id)
            .and_then(|i| i.as_scalar())
            .and_then(|i| i.to_integer())
            .unwrap_or(0);

        val -= 1;
        write!(writer, "{}", val).chain("Failed to render")?;
        context
            .stack_mut()
            .set_index(self.id.to_owned(), Value::scalar(val));
        Ok(())
    }
}

pub fn decrement_tag(
    _tag_name: &str,
    arguments: &[Token],
    _options: &LiquidOptions,
) -> Result<Box<Renderable>> {
    let mut args = arguments.iter();
    let id = match args.next() {
        Some(&Token::Identifier(ref id)) => id.clone(),
        x => return Err(unexpected_token_error("identifier", x)),
    };

    Ok(Box::new(Decrement { id }))
}

#[cfg(test)]
mod test {
    use super::*;
    use compiler;
    use interpreter;
    use tags;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options
            .tags
            .insert("assign", (tags::assign_tag as compiler::FnParseTag).into());
        options
            .tags
            .insert("increment", (increment_tag as compiler::FnParseTag).into());
        options
            .tags
            .insert("decrement", (decrement_tag as compiler::FnParseTag).into());
        options
    }

    #[test]
    fn increment() {
        let text = "{% increment val %}{{ val }}";
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "01");
    }

    #[test]
    fn decrement() {
        let text = "{% decrement val %}{{ val }}";
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "-1-1");
    }

    #[test]
    fn increment_and_decrement() {
        let text = "{% increment val %}{% increment val %}{% decrement val %}{% decrement val %}";
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "0110");
    }

    #[test]
    fn assign_and_increment() {
        let text = "{%- assign val = 9 -%}{% increment val %}{% increment val %}{{ val }}";
        let tokens = compiler::tokenize(text).unwrap();
        let options = options();
        let template = compiler::parse(&tokens, &options)
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "019");
    }
}
