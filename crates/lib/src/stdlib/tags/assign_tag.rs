use std::io::Write;

use liquid_core::error::ResultLiquidExt;
use liquid_core::parser::FilterChain;
use liquid_core::Blankness;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueView;
use liquid_core::{Error, Expression, ParseTag, TagReflection, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct AssignTag;

impl AssignTag {
    pub fn new() -> Self {
        Self
    }
}

impl TagReflection for AssignTag {
    fn tag(&self) -> &'static str {
        "assign"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for AssignTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let dst = arguments
            .expect_next("Identifier expected.")?
            .expect_identifier()
            .into_result()?
            .to_owned()
            .into();

        arguments
            .expect_next("Assignment operator \"=\" expected.")?
            .expect_str("=")
            .into_result_custom_msg("Assignment operator \"=\" expected.")?;

        let src_token = arguments.expect_next("FilterChain expected.")?;
        let src = match src_token.expect_range() {
            liquid_core::parser::TryMatchToken::Matches((start, stop)) => {
                AssignSource::Range(start, stop)
            }
            liquid_core::parser::TryMatchToken::Fails(token) => {
                AssignSource::Filter(token.expect_filter_chain(options).into_result()?)
            }
        };

        // no more arguments should be supplied, trying to supply them is an error
        arguments.expect_nothing()?;

        Ok(Box::new(Assign { dst, src }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Assign {
    dst: liquid_core::model::KString,
    src: AssignSource,
}

impl Assign {
    fn trace(&self) -> String {
        format!("{{% assign {} = {}%}}", self.dst, self.src)
    }
}

impl Renderable for Assign {
    fn render_to(&self, _writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let (value, range_bounds, assign_score) = match &self.src {
            AssignSource::Range(start, stop) => {
                let start = int_argument(start, runtime, "start")? as i64;
                let stop = int_argument(stop, runtime, "end")? as i64;
                (None, Some((start, stop)), 1)
            }
            AssignSource::Filter(chain) => {
                let (value, preserved_identity) = chain
                    .evaluate_with_identity(runtime)
                    .trace_with(|| self.trace().into())?;
                if preserved_identity {
                    if let Some(variable) = chain.as_plain_variable() {
                        let path = variable
                            .evaluate(runtime)
                            .trace_with(|| self.trace().into())?;
                        if runtime.set_global_alias(self.dst.clone(), path.as_slice()) {
                            let range_bounds =
                                copied_range_bounds(chain, runtime, preserved_identity)?;
                            let assign_score = range_bounds.map(|_| 1).unwrap_or(1);
                            (None, range_bounds, assign_score)
                        } else {
                            let value = value.into_owned();
                            let range_bounds =
                                copied_range_bounds(chain, runtime, preserved_identity)?;
                            let assign_score = range_bounds.map(|_| 1).unwrap_or_else(|| {
                                liquid_core::runtime::assign_resource_cost(runtime, &value)
                            });
                            (
                                range_bounds.map(|_| ()).map_or(Some(value), |_| None),
                                range_bounds,
                                assign_score,
                            )
                        }
                    } else {
                        let value = value.into_owned();
                        let range_bounds = copied_range_bounds(chain, runtime, preserved_identity)?;
                        let assign_score = range_bounds.map(|_| 1).unwrap_or_else(|| {
                            liquid_core::runtime::assign_resource_cost(runtime, &value)
                        });
                        (
                            range_bounds.map(|_| ()).map_or(Some(value), |_| None),
                            range_bounds,
                            assign_score,
                        )
                    }
                } else {
                    let value = value.into_owned();
                    let range_bounds = copied_range_bounds(chain, runtime, preserved_identity)?;
                    let assign_score = range_bounds.map(|_| 1).unwrap_or_else(|| {
                        liquid_core::runtime::assign_resource_cost(runtime, &value)
                    });
                    (
                        range_bounds.map(|_| ()).map_or(Some(value), |_| None),
                        range_bounds,
                        assign_score,
                    )
                }
            }
        };

        liquid_core::runtime::increment_assign_bytes(runtime, assign_score)?;
        if let Some((start, stop)) = range_bounds {
            runtime.set_global_range(self.dst.clone(), start, stop);
        } else if let Some(value) = value {
            runtime.set_global(self.dst.clone(), value);
        }
        Ok(())
    }

    fn blankness(&self) -> Blankness {
        Blankness::BlankNode
    }
}

#[derive(Debug)]
enum AssignSource {
    Filter(FilterChain),
    Range(Expression, Expression),
}

impl std::fmt::Display for AssignSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignSource::Filter(chain) => write!(f, "{chain}"),
            AssignSource::Range(start, stop) => write!(f, "({start}..{stop})"),
        }
    }
}

fn int_argument(arg: &Expression, runtime: &dyn Runtime, arg_name: &str) -> Result<isize> {
    let value = arg.evaluate(runtime)?;

    let value = value
        .as_scalar()
        .and_then(|value| value.to_integer())
        .ok_or_else(|| unexpected_value_error("whole number", Some(value.type_name())))
        .context_key_with(|| arg_name.to_owned().into())
        .value_with(|| value.to_kstr().into_owned())?;

    Ok(value as isize)
}

fn unexpected_value_error(expected: &str, actual: Option<&str>) -> Error {
    let actual = actual.unwrap_or("unknown");
    Error::with_msg(format!("Expected {expected}, found {actual}"))
}

fn copied_range_bounds(
    chain: &FilterChain,
    runtime: &dyn Runtime,
    preserved_identity: bool,
) -> Result<Option<(i64, i64)>> {
    if !preserved_identity {
        return Ok(None);
    }

    let Some(variable) = chain.source_variable() else {
        return Ok(None);
    };

    let path = variable.evaluate(runtime)?;
    if path.len() != 1 {
        return Ok(None);
    }

    let key = path[0].value().to_kstr();
    Ok(runtime.get_global_range_bounds(key.as_str()))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    use liquid_core::model::Scalar;
    use liquid_core::model::Value;
    use liquid_core::parser;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::runtime::{self, Runtime};

    use crate::stdlib;

    fn options() -> Language {
        let mut options = Language::default();
        options.tags.register("assign".to_owned(), AssignTag.into());
        std::sync::Arc::get_mut(&mut options.filters)
            .expect("default filter registry is uniquely owned")
            .register("default".to_owned(), stdlib::Default.into());
        std::sync::Arc::get_mut(&mut options.filters)
            .expect("default filter registry is uniquely owned")
            .register("reverse".to_owned(), stdlib::Reverse.into());
        options
            .blocks
            .register("if".to_owned(), stdlib::IfBlock.into());
        options
            .blocks
            .register("for".to_owned(), stdlib::ForBlock.into());
        options
    }

    struct RecordingRuntime {
        inner: Box<dyn Runtime>,
        last_global: RefCell<Option<Value>>,
    }

    impl RecordingRuntime {
        fn new() -> Self {
            Self {
                inner: Box::new(RuntimeBuilder::new().build()),
                last_global: RefCell::new(None),
            }
        }
    }

    impl Runtime for RecordingRuntime {
        fn partials(&self) -> &dyn runtime::PartialStore {
            self.inner.partials()
        }

        fn name(&self) -> Option<liquid_core::model::KStringRef<'_>> {
            self.inner.name()
        }

        fn roots(&self) -> std::collections::BTreeSet<liquid_core::model::KStringCow<'_>> {
            self.inner.roots()
        }

        fn try_get(
            &self,
            path: &[liquid_core::model::PathElement<'_>],
        ) -> Option<liquid_core::model::ValueCow<'_>> {
            self.inner.try_get(path)
        }

        fn get(
            &self,
            path: &[liquid_core::model::PathElement<'_>],
        ) -> Result<liquid_core::model::ValueCow<'_>> {
            self.inner.get(path)
        }

        fn set_global(&self, name: liquid_core::model::KString, val: Value) -> Option<Value> {
            self.last_global.borrow_mut().replace(val.clone());
            self.inner.set_global(name, val)
        }

        fn set_global_range(
            &self,
            name: liquid_core::model::KString,
            start: i64,
            stop: i64,
        ) -> Option<Value> {
            self.last_global.borrow_mut().replace(Value::Nil);
            self.inner.set_global_range(name, start, stop)
        }

        fn set_global_alias(
            &self,
            name: liquid_core::model::KString,
            source: &[liquid_core::model::PathElement<'_>],
        ) -> bool {
            self.inner.set_global_alias(name, source)
        }

        fn set_index(&self, name: liquid_core::model::KString, val: Value) -> Option<Value> {
            self.inner.set_index(name, val)
        }

        fn get_index<'a>(&'a self, name: &str) -> Option<liquid_core::model::ValueCow<'a>> {
            self.inner.get_index(name)
        }

        fn get_global_range_bounds(&self, name: &str) -> Option<(i64, i64)> {
            self.inner.get_global_range_bounds(name)
        }

        fn registers(&self) -> &runtime::Registers {
            self.inner.registers()
        }
    }

    #[test]
    fn assign() {
        let options = options();
        let template = parser::parse("{% assign freestyle = false %}{{ freestyle }}", &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "false");
    }

    #[test]
    fn assign_array_indexing() {
        let text = concat!("{% assign freestyle = tags[1] %}", "{{ freestyle }}");
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();
        runtime.set_global(
            "tags".into(),
            Value::Array(vec![
                Value::scalar("alpha"),
                Value::scalar("beta"),
                Value::scalar("gamma"),
            ]),
        );

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "beta");
    }

    #[test]
    fn assign_object_indexing() {
        let text = concat!(
            r#"{% assign freestyle = tags["greek"] %}"#,
            "{{ freestyle }}"
        );
        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();
        runtime.set_global(
            "tags".into(),
            Value::Object(
                vec![("greek".into(), Value::scalar("alpha"))]
                    .into_iter()
                    .collect(),
            ),
        );

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "alpha");
    }

    #[test]
    fn assign_in_loop_persists_on_loop_exit() {
        let text = concat!(
            "{% assign freestyle = false %}",
            "{% for t in tags %}{% if t == 'freestyle' %}",
            "{% assign freestyle = true %}",
            "{% endif %}{% endfor %}",
            "{% if freestyle %}",
            "<p>Freestyle!</p>",
            "{% endif %}"
        );

        let options = options();
        let template = parser::parse(text, &options)
            .map(runtime::Template::new)
            .unwrap();

        // test one: no matching value in `tags`
        {
            let runtime = RuntimeBuilder::new().build();
            runtime.set_global(
                "tags".into(),
                Value::Array(vec![
                    Value::scalar("alpha"),
                    Value::scalar("beta"),
                    Value::scalar("gamma"),
                ]),
            );

            let output = template.render(&runtime).unwrap();
            assert_eq!(
                runtime.get(&[Scalar::new("freestyle").into()]).unwrap(),
                false
            );
            assert_eq!(output, "");
        }

        // test two: matching value in `tags`
        {
            let runtime = RuntimeBuilder::new().build();
            runtime.set_global(
                "tags".into(),
                Value::Array(vec![
                    Value::scalar("alpha"),
                    Value::scalar("beta"),
                    Value::scalar("freestyle"),
                    Value::scalar("gamma"),
                ]),
            );

            let output = template.render(&runtime).unwrap();
            assert_eq!(
                runtime.get(&[Scalar::new("freestyle").into()]).unwrap(),
                true
            );
            assert_eq!(output, "<p>Freestyle!</p>");
        }
    }

    #[test]
    fn assign_range_through_default_preserves_range_identity() {
        let options = options();
        let template = parser::parse(
            "{% assign foo = (1..5) %}{% assign bar = foo | default: nil %}{{ bar }}|{{ bar.size }}",
            &options,
        )
        .map(runtime::Template::new)
        .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "1..5|5");
    }

    #[test]
    fn assign_range_through_non_identity_filters_drops_range_identity() {
        let options = options();
        let template = parser::parse(
            "{% assign foo = (1..5) %}{% assign bar = foo | reverse | reverse %}{{ bar }}|{{ bar.size }}",
            &options,
        )
        .map(runtime::Template::new)
        .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "12345|5");
    }

    #[test]
    fn assign_range_literal_uses_lightweight_global_storage() {
        let options = options();
        let template = parser::parse("{% assign foo = (1..1000) %}{{ foo.size }}", &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RecordingRuntime::new();

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "1000");
        assert_eq!(*runtime.last_global.borrow(), Some(Value::Nil));
    }
}
