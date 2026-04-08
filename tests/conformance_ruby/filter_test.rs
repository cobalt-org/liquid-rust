use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{Display_filter, Filter, FilterReflection, ParseFilter};
use liquid_core::{Value, ValueView};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(name = "money", description = "tests helper", parsed(MoneyFilter))]
pub(super) struct MoneyFilterParser;

#[derive(Debug, Default, Display_filter)]
#[name = "money"]
pub(super) struct MoneyFilter;

impl Filter for MoneyFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        Ok(Value::scalar(format!(" {}$ ", input.render())))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "money_with_underscore",
    description = "tests helper",
    parsed(MoneyWithUnderscoreFilter)
)]
pub(super) struct MoneyWithUnderscoreFilterParser;

#[derive(Debug, Default, Display_filter)]
#[name = "money_with_underscore"]
pub(super) struct MoneyWithUnderscoreFilter;

impl Filter for MoneyWithUnderscoreFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        Ok(Value::scalar(format!(" {}$ ", input.render())))
    }
}

fn liquid_money() -> liquid::Parser {
    liquid::ParserBuilder::with_stdlib()
        .filter(MoneyFilterParser)
        .filter(MoneyWithUnderscoreFilterParser)
        .build()
        .unwrap()
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "substitute",
    description = "tests helper",
    parsed(SubstituteFilter)
)]
pub(super) struct SubstituteFilterParser;

#[derive(Debug, Default, Display_filter)]
#[name = "substitute"]
pub(super) struct SubstituteFilter;

impl Filter for SubstituteFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        Ok(Value::scalar(format!(
            "No keyword argument support: {}",
            input.render()
        )))
    }
}

fn liquid_sub() -> liquid::Parser {
    liquid::ParserBuilder::with_stdlib()
        .filter(SubstituteFilterParser)
        .build()
        .unwrap()
}

#[test]
fn test_local_filter() {
    let assigns = o!({"var": 1000});

    assert_template_result!(" 1000$ ", "{{var | money}}", assigns, liquid_money());
}

#[test]
fn test_underscore_in_filter_name() {
    let assigns = o!({"var": 1000});

    assert_template_result!(
        " 1000$ ",
        "{{var | money_with_underscore}}",
        assigns,
        liquid_money()
    );
}

#[test]
#[should_panic]
fn test_second_filter_overwrites_first() {
    panic!("Implementation specific: API for adding filters");
}

#[test]
fn test_size() {
    let assigns = o!({"var": "abcd"});

    assert_template_result!("4", "{{var | size}}", assigns);
}

#[test]
fn test_join() {
    let assigns = o!({"var": [1, 2, 3, 4]});

    assert_template_result!("1 2 3 4", "{{var | join}}", assigns);
}

#[test]
fn test_sort() {
    let assigns = o!({
        "value": 3,
        "numbers": [2, 1, 4, 3],
        "words": ["expected", "as", "alphabetic"],
        "arrays": ["flower", "are"],
        "case_sensitive": ["sensitive", "Expected", "case"],
    });

    assert_template_result!("1 2 3 4", "{{numbers | sort | join}}", assigns);
    assert_template_result!("alphabetic as expected", "{{words | sort | join}}", assigns);
    assert_template_result!("3", "{{value | sort}}", assigns);
    assert_template_result!("are flower", "{{arrays | sort | join}}", assigns);
    assert_template_result!(
        "Expected case sensitive",
        "{{case_sensitive | sort | join}}",
        assigns
    );
}

#[test]
fn test_sort_natural() {
    let assigns = o!({
        "words": ["case", "Assert", "Insensitive"],
        "hashes": [{ "a": "A" }, { "a": "b" }, { "a": "C" }],
    });

    // Test strings
    assert_template_result!(
        "Assert case Insensitive",
        "{{words | sort_natural | join}}",
        assigns
    );

    // Test hashes
    assert_template_result!(
        "A b C",
        "{{hashes | sort_natural: 'a' | map: 'a' | join}}",
        assigns
    );

    // Test objects
    // Implementation specific: API support objects for variables.
}

#[test]
fn test_compact() {
    let assigns = o!({
        "words": ["a", nil, "b", nil, "c"],
        "hashes": [{ "a": "A" }, { "a": nil }, { "a": "C" }],
    });

    // Test strings
    assert_template_result!("a b c", "{{words | compact | join}}", assigns);

    // Test hashes
    assert_template_result!(
        "A C",
        "{{hashes | compact: 'a' | map: 'a' | join}}",
        assigns
    );

    // Test objects
    // Implementation specific: API support objects for variables.
}

#[test]
fn test_strip_html() {
    let assigns = o!({"var": "<b>bla blub</a>"});

    assert_template_result!("bla blub", "{{var | strip_html }}", assigns);
}

#[test]
fn test_strip_html_ignore_comments_with_html() {
    let assigns = o!({"var": "<!-- split and some <ul> tag --><b>bla blub</a>"});

    assert_template_result!("bla blub", "{{var | strip_html }}", assigns);
}

#[test]
fn test_capitalize() {
    let assigns = o!({"var": "blub"});

    assert_template_result!("Blub", "{{var | capitalize }}", assigns);
}

#[test]
#[should_panic]
fn test_nonexistent_filter_is_ignored() {
    panic!("Implementation specific: strict_filters");
}

#[test]
#[should_panic] // liquid-rust#92
fn test_filter_with_keyword_arguments() {
    let assigns = o!({
        "surname": "john",
        "input": "hello %{first_name}, %{last_name}",
    });
    assert_template_result!(
        "hello john, doe",
        "{{ input | substitute: first_name: surname, last_name: 'doe' }}",
        assigns,
        liquid_sub()
    );
}

#[test]
fn test_filter_argument_errors_are_deferred_to_render() {
    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("{{ '' | size: 'too many args' }}").unwrap();
    let rendered = template.render(&liquid_core::Object::new()).unwrap();

    assert!(rendered.contains("wrong number of arguments"));
}

#[test]
#[should_panic]
fn test_override_object_method_in_filter() {
    panic!("Implementation specific: object API");
}

#[test]
#[should_panic]
fn test_local_global() {
    panic!("Implementation specific: local/global API");
}

#[test]
#[should_panic]
fn test_local_filter_with_deprecated_syntax() {
    panic!("Implementation specific: local/global API");
}

#[cfg(feature = "conformance-harness")]
mod conformance_harness_tests {
    use std::rc::Rc;

    use liquid::conformance::{self, ConformanceCallbacks, FallbackFilterResolver, RenderConfig};
    use liquid_core::parser::FilterCall;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::{Error, Result, Runtime, Value, ValueView};

    struct NoopCallbacks;

    impl ConformanceCallbacks for NoopCallbacks {
        fn handle_render_error(
            &self,
            _runtime: &dyn Runtime,
            error: Error,
        ) -> Result<Option<String>> {
            Err(error)
        }

        fn increment_render_ops(&self, _amount: usize) -> Result<()> {
            Ok(())
        }

        fn increment_assign_bytes(&self, _amount: usize) -> Result<()> {
            Ok(())
        }

        fn check_resource_limits(
            &self,
            _runtime: &dyn Runtime,
            _rendered_bytes: usize,
        ) -> Result<()> {
            Ok(())
        }

        fn reset_resource_limits(&self) -> Result<()> {
            Ok(())
        }
    }

    struct LateFilterResolver;

    impl FallbackFilterResolver for LateFilterResolver {
        fn has_filter(&self, name: &str) -> bool {
            name == "late_money"
        }

        fn evaluate(
            &self,
            _filter: &FilterCall,
            input: &dyn ValueView,
            _runtime: &dyn Runtime,
        ) -> Result<Value> {
            Ok(Value::scalar(format!("late:{}", input.render())))
        }
    }

    struct OverrideDeferredErrorFilterResolver;

    impl FallbackFilterResolver for OverrideDeferredErrorFilterResolver {
        fn has_filter(&self, name: &str) -> bool {
            name == "size"
        }

        fn evaluate(
            &self,
            _filter: &FilterCall,
            input: &dyn ValueView,
            _runtime: &dyn Runtime,
        ) -> Result<Value> {
            Ok(Value::scalar(format!("override:{}", input.render())))
        }
    }

    #[test]
    fn test_late_registered_filter_dispatches_through_conformance_resolver() {
        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        let template =
            conformance::parse("{{ price | late_money }}", parser.conformance_language()).unwrap();
        let globals = o!({ "price": 42 });
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let mut output = Vec::new();

        conformance::render_to(
            &template,
            &mut output,
            &runtime,
            &RenderConfig {
                strict_variables: false,
                strict_filters: true,
                callbacks: Rc::new(NoopCallbacks),
                fallback_filters: Some(Rc::new(LateFilterResolver)),
                live_scope_session: None,
            },
        )
        .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "late:42");
    }

    #[test]
    fn test_deferred_filter_error_can_dispatch_through_conformance_resolver() {
        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        let template =
            conformance::parse("{{ price | size: 'too many args' }}", parser.conformance_language())
                .unwrap();
        let globals = o!({ "price": 42 });
        let runtime = RuntimeBuilder::new().set_globals(&globals).build();
        let mut output = Vec::new();

        conformance::render_to(
            &template,
            &mut output,
            &runtime,
            &RenderConfig {
                strict_variables: false,
                strict_filters: true,
                callbacks: Rc::new(NoopCallbacks),
                fallback_filters: Some(Rc::new(OverrideDeferredErrorFilterResolver)),
                live_scope_session: None,
            },
        )
        .unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "override:42");
    }
}
