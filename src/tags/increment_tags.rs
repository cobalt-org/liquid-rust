use std::io::Write;

use liquid_error::{Result, ResultLiquidReplaceExt};

use compiler::Language;
use compiler::ParseTag;
use compiler::TagReflection;
use compiler::TagTokenIter;
use interpreter::Context;
use interpreter::Renderable;
use value::Value;

#[derive(Clone, Debug)]
struct Increment {
    id: String,
}

impl Renderable for Increment {
    fn render_to(&self, writer: &mut dyn Write, context: &mut Context) -> Result<()> {
        let mut val = context
            .stack()
            .get_index(&self.id)
            .and_then(|i| i.as_scalar())
            .and_then(|i| i.to_integer())
            .unwrap_or(0);

        write!(writer, "{}", val).replace("Failed to render")?;
        val += 1;
        context
            .stack_mut()
            .set_index(self.id.to_owned(), Value::scalar(val));
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct IncrementTag;

impl IncrementTag {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TagReflection for IncrementTag {
    fn tag(&self) -> &'static str {
        "increment"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for IncrementTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let id = arguments
            .expect_next("Identifier expected.")?
            .expect_identifier()
            .into_result()?
            .to_string();

        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        Ok(Box::new(Increment { id }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Clone, Debug)]
struct Decrement {
    id: String,
}

impl Renderable for Decrement {
    fn render_to(&self, writer: &mut dyn Write, context: &mut Context) -> Result<()> {
        let mut val = context
            .stack()
            .get_index(&self.id)
            .and_then(|i| i.as_scalar())
            .and_then(|i| i.to_integer())
            .unwrap_or(0);

        val -= 1;
        write!(writer, "{}", val).replace("Failed to render")?;
        context
            .stack_mut()
            .set_index(self.id.to_owned(), Value::scalar(val));
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DecrementTag;

impl DecrementTag {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TagReflection for DecrementTag {
    fn tag(&self) -> &'static str {
        "decrement"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for DecrementTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let id = arguments
            .expect_next("Identifier expected.")?
            .expect_identifier()
            .into_result()?
            .to_string();

        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        Ok(Box::new(Decrement { id }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
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
        options.tags.register("assign", tags::AssignTag.into());
        options.tags.register("increment", IncrementTag.into());
        options.tags.register("decrement", DecrementTag.into());
        options
    }

    #[test]
    fn increment() {
        let text = "{% increment val %}{{ val }}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "01");
    }

    #[test]
    fn decrement() {
        let text = "{% decrement val %}{{ val }}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "-1-1");
    }

    #[test]
    fn increment_and_decrement() {
        let text = "{% increment val %}{% increment val %}{% decrement val %}{% decrement val %}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "0110");
    }

    #[test]
    fn assign_and_increment() {
        let text = "{%- assign val = 9 -%}{% increment val %}{% increment val %}{{ val }}";
        let template = compiler::parse(text, &options())
            .map(interpreter::Template::new)
            .unwrap();

        let mut context = Context::new();
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, "019");
    }
}
