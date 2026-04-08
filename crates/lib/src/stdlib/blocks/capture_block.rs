use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::model::Value;
use liquid_core::Blankness;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::Template;
use liquid_core::{BlockReflection, ParseBlock, TagBlock, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct CaptureBlock;

impl CaptureBlock {
    pub fn new() -> Self {
        Self
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
        let id = parse_capture_target(arguments.expect_next("Identifier expected")?)?;

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

fn parse_capture_target(
    token: liquid_core::parser::TagToken<'_>,
) -> Result<liquid_core::model::KString> {
    let raw = token.as_str();
    let candidate = raw
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            raw.strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
        .unwrap_or(raw);

    if !candidate.is_empty()
        && candidate.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '[' | ']' | '(' | ')')
        })
    {
        Ok(candidate.to_owned().into())
    } else {
        token.raise_custom_error("Identifier expected").into_err()
    }
}

#[derive(Debug)]
struct Capture {
    id: liquid_core::model::KString,
    template: Template,
}

impl Capture {
    fn trace(&self) -> String {
        format!("{{% capture {} %}}", self.id)
    }
}

impl Renderable for Capture {
    fn render_to(&self, _writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let mut captured = Vec::new();
        self.template
            .render_to(&mut captured, runtime)
            .trace_with(|| self.trace().into())?;

        liquid_core::runtime::increment_assign_bytes(runtime, captured.len())?;
        let output = String::from_utf8(captured).expect("render only writes UTF-8");
        runtime.set_global(self.id.clone(), Value::scalar(output));
        Ok(())
    }

    fn blankness(&self) -> Blankness {
        Blankness::BlankNode
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::model::Scalar;
    use liquid_core::parser;
    use liquid_core::runtime::RuntimeBuilder;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("assign".to_owned(), crate::stdlib::AssignTag.into());
        options
            .blocks
            .register("capture".to_owned(), CaptureBlock.into());
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
        let template = parser::parse(text, &options).map(Template::new).unwrap();

        let rt = RuntimeBuilder::new().build();
        rt.set_global("item".into(), Value::scalar("potato"));
        rt.set_global("i".into(), Value::scalar(42f64));

        let output = template.render(&rt).unwrap();
        assert_eq!(
            rt.get(&[Scalar::new("attribute_name").into()]).unwrap(),
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
        let template = parser::parse(text, &options).map(Template::new);
        assert!(template.is_err());
    }

    #[test]
    fn capture_accepts_quoted_variable_names() {
        let text = concat!(
            "{% capture 'var' %}",
            "test string",
            "{% endcapture %}",
            "{{var}}"
        );
        let options = options();
        let template = parser::parse(text, &options).map(Template::new).unwrap();

        let rt = RuntimeBuilder::new().build();
        assert_eq!(template.render(&rt).unwrap(), "test string");
    }

    #[test]
    fn capture_accepts_hyphenated_variable_names() {
        let text = concat!(
            "{% capture this-thing %}",
            "Print this-thing",
            "{% endcapture %}",
            "{{ this-thing }}"
        );
        let options = options();
        let template = parser::parse(text, &options).map(Template::new).unwrap();

        let rt = RuntimeBuilder::new().build();
        assert_eq!(template.render(&rt).unwrap(), "Print this-thing");
    }

    #[test]
    fn capture_overwrites_previous_range_assignment() {
        let text = concat!(
            "{% assign captured = (1..3) %}",
            "{% capture captured %}done{% endcapture %}",
            "{{ captured }}"
        );
        let options = options();
        let template = parser::parse(text, &options).map(Template::new).unwrap();

        let rt = RuntimeBuilder::new().build();
        assert_eq!(template.render(&rt).unwrap(), "done");
    }
}
