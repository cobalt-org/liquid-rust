use std::borrow;

use liquid;

#[derive(Default, Debug, Clone, Copy)]
struct TestFileSystem;

impl liquid::partials::PartialSource for TestFileSystem {
    fn contains(&self, _name: &str) -> bool {
        true
    }

    fn names(&self) -> Vec<&str> {
        vec![]
    }

    fn try_get<'a>(&'a self, name: &str) -> Option<borrow::Cow<'a, str>> {
        let template = match name {
            "product" => "Product: {{ product.title }} ".into(),
            "locale_variables" => "Locale: {{echo1}} {{echo2}}".into(),
            "variant" => "Variant: {{ variant.title }}".into(),
            "nested_template" => {
                "{% include 'header' %} {% include 'body' %} {% include 'footer' %}".into()
            }
            "body" => "body {% include 'body_detail' %}".into(),
            "nested_product_template" => {
                "Product: {{ nested_product_template.title }} {%include 'details'%} ".into()
            }
            "recursively_nested_template" => "-{% include 'recursively_nested_template' %}".into(),
            "pick_a_source" => "from TestFileSystem".into(),
            "assignments" => "{% assign foo = 'bar' %}".into(),
            _ => name.to_owned().into(),
        };
        Some(template)
    }
}

fn liquid() -> liquid::Parser {
    liquid::ParserBuilder::with_liquid()
        .partials(liquid::partials::OnDemandCompiler::<TestFileSystem>::empty())
        .build()
        .unwrap()
}

#[test]
#[should_panic]
fn test_include_tag_looks_for_file_system_in_registers_first() {
    panic!("Implementation specific: exposing of registers API");
}

#[test]
#[should_panic] // liquid-rust#237
fn test_include_tag_with() {
    assert_template_result!(
        "Product: Draft 151cm ",
        "{% include 'product' with products[0] %}",
        v!({"products": [ { "title": "Draft 151cm" }, { "title": "Element 155cm" } ]}),
        liquid()
    );
}

#[test]
fn test_include_tag_with_default_name() {
    assert_template_result!(
        "Product: Draft 151cm ",
        "{% include 'product' %}",
        v!({"product": { "title": "Draft 151cm" }}),
        liquid()
    );
}

#[test]
#[should_panic] // liquid-rust#237
fn test_include_tag_for() {
    assert_template_result!(
        "Product: Draft 151cm Product: Element 155cm ",
        "{% include 'product' for products %}",
        v!({"products": [ { "title": "Draft 151cm" }, { "title": "Element 155cm" } ]}),
        liquid()
    );
}

#[test]
#[should_panic] // liquid-rust#237
fn test_include_tag_with_local_variables() {
    assert_template_result!(
        "Locale: test123 ",
        "{% include 'locale_variables' echo1: 'test123' %}",
        v!({}),
        liquid()
    );
}

#[test]
#[should_panic] // liquid-rust#237
fn test_include_tag_with_multiple_local_variables() {
    assert_template_result!(
        "Locale: test123 test321",
        "{% include 'locale_variables' echo1: 'test123', echo2: 'test321' %}",
        v!({}),
        liquid()
    );
}

#[test]
#[should_panic] // liquid-rust#237
fn test_include_tag_with_multiple_local_variables_from_context() {
    assert_template_result!(
        "Locale: test123 test321",
        "{% include 'locale_variables' echo1: echo1, echo2: more_echos.echo2 %}",
        v!({"echo1": "test123", "more_echos": { "echo2": "test321" }}),
        liquid()
    );
}

#[test]
fn test_included_templates_assigns_variables() {
    assert_template_result!(
        "bar",
        "{% include 'assignments' %}{{ foo }}",
        v!({}),
        liquid()
    );
}

#[test]
fn test_nested_include_tag() {
    assert_template_result!("body body_detail", "{% include 'body' %}", v!({}), liquid());

    assert_template_result!(
        "header body body_detail footer",
        "{% include 'nested_template' %}",
        v!({}),
        liquid()
    );
}

#[test]
#[should_panic] // liquid-rust#237
fn test_nested_include_with_variable() {
    assert_template_result!(
        "Product: Draft 151cm details ",
        "{% include 'nested_product_template' with product %}",
        v!({"product": { "title": "Draft 151cm" }}),
        liquid()
    );

    assert_template_result!(
        "Product: Draft 151cm details Product: Element 155cm details ",
        "{% include 'nested_product_template' for products %}",
        v!({"products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }]}),
        liquid()
    );
}

#[derive(Default, Debug, Clone, Copy)]
struct InfiniteFileSystem;

impl liquid::partials::PartialSource for InfiniteFileSystem {
    fn contains(&self, _name: &str) -> bool {
        true
    }

    fn names(&self) -> Vec<&str> {
        vec![]
    }

    fn try_get<'a>(&'a self, _name: &str) -> Option<borrow::Cow<'a, str>> {
        Some("-{% include 'loop' %}".into())
    }
}

#[test]
#[should_panic] // liquid-rust#300
fn test_recursively_included_template_does_not_produce_endless_loop() {
    panic!("We don't check recursion depth");
    /*
    liquid::ParserBuilder::with_liquid()
        .partials(liquid::partials::OnDemandCompiler::<TestFileSystem>::empty())
        .build()
        .unwrap()
    parser.parse("{% include 'loop' %}").unwrap();
    */
}

#[test]
#[should_panic] // liquid-rust#275
fn test_dynamically_choosen_template() {
    assert_template_result!(
        "Test123",
        "{% include template %}",
        v!({"template": "Test123"}),
        liquid()
    );
    assert_template_result!(
        "Test321",
        "{% include template %}",
        v!({"template": "Test321"}),
        liquid()
    );

    assert_template_result!(
        "Product: Draft 151cm ",
        "{% include template for product %}",
        v!({"template": "product", "product": { "title": "Draft 151cm" }}),
        liquid()
    );
}

#[test]
#[should_panic]
fn test_include_tag_caches_second_read_of_same_partial() {
    panic!("Implementation specific: caching policies");
}

#[test]
#[should_panic]
fn test_include_tag_doesnt_cache_partials_across_renders() {
    panic!("Implementation specific: caching policies");
}

#[test]
fn test_include_tag_within_if_statement() {
    assert_template_result!(
        "foo_if_true",
        "{% if true %}{% include 'foo_if_true' %}{% endif %}",
        v!({}),
        liquid()
    );
}

#[test]
#[should_panic]
fn test_custom_include_tag() {
    panic!("Implementation specific: API customization");
}

#[test]
#[should_panic]
fn test_custom_include_tag_within_if_statement() {
    panic!("Implementation specific: API customization");
}

#[test]
fn test_does_not_add_error_in_strict_mode_for_missing_variable() {
    let template = liquid()
        .parse(r#" {% include "nested_template" %}"#)
        .unwrap();
    template.render(v!({}).as_object().unwrap()).unwrap();
}

#[test]
#[should_panic]
fn test_passing_options_to_included_templates() {
    panic!("Implementation specific: API options");
}

#[test]
#[should_panic] // liquid-rust#275
fn test_render_raise_argument_error_when_template_is_undefined() {
    assert_parse_error!("{% include undefined_variable %}", liquid());
    assert_parse_error!("{% include nil %}", liquid());
}

#[test]
#[should_panic] // liquid-rust#275
fn test_including_via_variable_value() {
    assert_template_result!(
        "from TestFileSystem",
        "{% assign page = 'pick_a_source' %}{% include page %}",
        v!({}),
        liquid()
    );

    assert_template_result!(
        "Product: Draft 151cm ",
        "{% assign page = 'product' %}{% include page %}",
        v!({"product": { "title": "Draft 151cm" }}),
        liquid()
    );

    assert_template_result!(
        "Product: Draft 151cm ",
        "{% assign page = 'product' %}{% include page for foo %}",
        v!({"foo": { "title": "Draft 151cm" }}),
        liquid()
    );
}

#[test]
fn test_including_with_strict_variables() {
    let template = liquid().parse("{% include 'simple' %}").unwrap();
    template.render(v!({}).as_object().unwrap()).unwrap();
}
