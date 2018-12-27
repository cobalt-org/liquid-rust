use liquid;
use liquid::compiler::FilterResult;
use liquid::value::Value;

fn money(input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar(format!(" {}$ ", input.render())))
}

fn money_with_underscore(input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar(format!(" {}$ ", input.render())))
}

fn liquid_money() -> liquid::Parser {
    liquid::ParserBuilder::with_liquid()
        .filter("money", money as liquid::compiler::FnFilterValue)
        .filter(
            "money_with_underscore",
            money_with_underscore as liquid::compiler::FnFilterValue,
        )
        .build()
        .unwrap()
}

fn substitute(input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar(format!(
        "No keyword argument support: {}",
        input.render()
    )))
}

fn liquid_sub() -> liquid::Parser {
    liquid::ParserBuilder::with_liquid()
        .filter("substitute", substitute as liquid::compiler::FnFilterValue)
        .build()
        .unwrap()
}

#[test]
fn test_local_filter() {
    let assigns = v!({"var": 1000});

    assert_template_result!(" 1000$ ", "{{var | money}}", assigns, liquid_money());
}

#[test]
fn test_underscore_in_filter_name() {
    let assigns = v!({"var": 1000});

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
    let assigns = v!({"var": "abcd"});

    assert_template_result!("4", "{{var | size}}", assigns);
}

#[test]
fn test_join() {
    let assigns = v!({"var": [1, 2, 3, 4]});

    assert_template_result!("1 2 3 4", "{{var | join}}", assigns);
}

#[test]
#[should_panic] // liquid-rust#250
fn test_sort() {
    let assigns = v!({
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
#[should_panic] // liquid-rust#249
fn test_sort_natural() {
    let assigns = v!({
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
#[should_panic] // liquid-rust#246
fn test_compact() {
    let assigns = v!({
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
    let assigns = v!({"var": "<b>bla blub</a>"});

    assert_template_result!("bla blub", "{{var | strip_html }}", assigns);
}

#[test]
fn test_strip_html_ignore_comments_with_html() {
    let assigns = v!({"var": "<!-- split and some <ul> tag --><b>bla blub</a>"});

    assert_template_result!("bla blub", "{{var | strip_html }}", assigns);
}

#[test]
fn test_capitalize() {
    let assigns = v!({"var": "blub"});

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
    let assigns = v!({
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
