use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::Value;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::Template;
use liquid_core::{BlockReflection, ParseBlock, TagBlock, TagTokenIter};

#[derive(Debug)]
struct Capture {
    id: String,
    template: Template,
}

impl Capture {
    fn trace(&self) -> String {
        format!("{{% capture {} %}}", self.id)
    }
}

impl Renderable for Capture {
    fn render_to(&self, _writer: &mut dyn Write, runtime: &mut Runtime<'_>) -> Result<()> {
        let mut captured = Vec::new();
        self.template
            .render_to(&mut captured, runtime)
            .trace_with(|| self.trace().into())?;

        let output = String::from_utf8(captured).expect("render only writes UTF-8");
        runtime
            .stack_mut()
            .set_global(self.id.to_owned(), Value::scalar(output));
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CaptureBlock;

impl CaptureBlock {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BlockReflection for CaptureBlock {
    fn start_tag(&self) -> &str {
        "capture"
    }

    fn end_tag(&self) -> &str {
        "endcapture"
    }

    fn description(&self) -> &str {
        ""
    }
}

impl ParseBlock for CaptureBlock {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        mut tokens: TagBlock<'_, '_>,
        options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let id = arguments
            .expect_next("Identifier expected")?
            .expect_identifier()
            .into_result()?
            .to_string();

        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        let template = Template::new(
            tokens
                .parse_all(options)
                .trace_with(|| format!("{{% capture {} %}}", &id).into())?,
        );

        tokens.assert_empty();
        Ok(Box::new(Capture { id, template }))
    }

    fn reflection(&self) -> &dyn BlockReflection {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::model::Scalar;
    use liquid_core::parser;
    use liquid_core::runtime;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .blocks
            .register("capture".to_string(), CaptureBlock.into());
        options
    }

    #[test]
    fn test_capture() {
        let text = concat!(
            "{% capture attribute_name %}",
            "{{ item }}-{{ i }}-color",
            "{% endcapture %}"
        );
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let mut rt = Runtime::new();
        rt.stack_mut().set_global("item", Value::scalar("potato"));
        rt.stack_mut().set_global("i", Value::scalar(42f64));

        let output = template.render(&mut rt).unwrap();
        assert_eq!(
            rt.stack().get(&[Scalar::new("attribute_name")]).unwrap(),
            "potato-42-color"
        );
        assert_eq!(output, "");
    }

    #[test]
    fn trailing_tokens_are_an_error() {
        let text = concat!(
            "{% capture foo bar baz %}",
            "We should never see this",
            "{% endcapture %}"
        );
        let options = options();
        let template = parser::parse(text, &options).map(runtime::Template::new);
        assert!(template.is_err());
    }
}
