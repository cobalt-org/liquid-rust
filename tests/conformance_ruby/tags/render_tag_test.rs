fn liquid(partials: liquid::Object) -> liquid::Parser {
    let mut source = liquid::partials::InMemorySource::new();
    for (key, value) in partials {
        source.add(key.as_str(), value.into_scalar().unwrap().into_cow_str());
    }
    let partials = liquid::partials::EagerCompiler::new(source);
    liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
}

#[test]
fn test_render_with_no_arguments() {
    assert_template_result!(
        "rendered content",
        "{% render 'source' %}",
        o!({}),
        liquid(o!({ "source": "rendered content" })),
    );
}

#[test]
fn test_render_tag_looks_for_file_system_in_registers_first() {
    assert_template_result!(
        "from register file system",
        "{% render 'pick_a_source' %}",
        o!({}),
        liquid(o!({ "pick_a_source": "from register file system" })),
    );
}

#[test]
fn test_render_passes_named_arguments_into_inner_scope() {
    assert_template_result!(
        "My Product",
        "{% render 'product', inner_product: outer_product %}",
        o!({ "outer_product": { "title": "My Product" } }),
        liquid(o!({ "product": "{{ inner_product.title }}" })),
    );
}

#[test]
fn test_render_accepts_literals_as_arguments() {
    assert_template_result!(
        "123",
        "{% render 'snippet', price: 123 %}",
        o!({}),
        liquid(o!({ "snippet": "{{ price }}" })),
    );
}

#[test]
fn test_render_accepts_multiple_named_arguments() {
    assert_template_result!(
        "1 2",
        "{% render 'snippet', one: 1, two: 2 %}",
        o!({}),
        liquid(o!({ "snippet": "{{ one }} {{ two }}" })),
    );
}

#[test]
fn test_render_does_not_inherit_parent_scope_variables() {
    assert_template_result!(
        "",
        "{% assign outer_variable = 'should not be visible' %}{% render 'snippet' %}",
        o!({}),
        liquid(o!({ "snippet": "{% if outer_variable %}broken{% endif %}" })),
    );
}

#[test]
fn test_render_does_not_inherit_variable_with_same_name_as_snippet() {
    assert_template_result!(
        "",
        "{% assign snippet = 'should not be visible' %}{% render 'snippet' %}",
        o!({}),
        liquid(o!({ "snippet": "{% if snippet %}broken{% endif %}" })),
    );
}

#[test]
fn test_render_does_not_mutate_parent_scope() {
    assert_template_result!(
        "",
        "{% render 'snippet' %}{% if inner %}broken{% endif %}",
        o!({}),
        liquid(o!({ "snippet": "{% assign inner = 1 %}" })),
    );
}

#[test]
fn test_nested_render_tag() {
    assert_template_result!(
        "one two",
        "{% render 'one' %}",
        o!({}),
        liquid(o!({
          "one": "one {% render 'two' %}",
          "two": "two",
        })),
    );
}

/*
#[test]
fn test_recursively_rendered_template_does_not_produce_endless_loop() {
  Liquid::Template.file_system = StubFileSystem.new("loop": "{% render 'loop' %}")

  assert_raises(Liquid::StackLevelError) do
    Template.parse("{% render 'loop' %}").render!
  end
}

#[test]
fn test_sub_contexts_count_towards_the_same_recursion_limit() {
  Liquid::Template.file_system = StubFileSystem.new(
    "loop_render": "{% render 'loop_render' %}",
  )
  assert_raises(Liquid::StackLevelError) do
    Template.parse("{% render 'loop_render' %}").render!
  end
}

#[test]
fn test_dynamically_choosen_templates_are_not_allowed() {
  assert_syntax_error("{% assign name = 'snippet' %}{% render name %}")
}

#[test]
fn test_include_tag_caches_second_read_of_same_partial() {
  file_system = StubFileSystem.new("snippet": "echo")
  assert_equal(
    "echoecho",
    Template.parse("{% render 'snippet' %}{% render 'snippet' %}")
    .render!({}, registers: { file_system: file_system }),
  )
  assert_equal(1, file_system.file_read_count)
}

#[test]
fn test_render_tag_doesnt_cache_partials_across_renders() {
  file_system = StubFileSystem.new("snippet": "my message")

  assert_equal(
    "my message",
    Template.parse("{% include 'snippet' %}").render!({}, registers: { file_system: file_system }),
  )
  assert_equal(1, file_system.file_read_count)

  assert_equal(
    "my message",
    Template.parse("{% include 'snippet' %}").render!({}, registers: { file_system: file_system }),
  )
  assert_equal(2, file_system.file_read_count)
}
*/

#[test]
fn test_render_tag_within_if_statement() {
    assert_template_result!(
        "my message",
        "{% if true %}{% render 'snippet' %}{% endif %}",
        o!({}),
        liquid(o!({ "snippet": "my message" })),
    );
}

#[test]
fn test_break_through_render() {
    let liquid = liquid(o!({ "break": "{% break %}" }));
    assert_template_result!(
        "1",
        "{% for i in (1..3) %}{{ i }}{% break %}{{ i }}{% endfor %}",
        o!({}),
        liquid
    );
    assert_template_result!(
        "112233",
        "{% for i in (1..3) %}{{ i }}{% render 'break' %}{{ i }}{% endfor %}",
        o!({}),
        liquid
    );
}

#[test]
#[should_panic] // `increment` without a variable is not supported yet
fn test_increment_is_isolated_between_renders() {
    assert_template_result!(
        "010",
        "{% increment %}{% increment %}{% render 'incr' %}",
        o!({}),
        liquid(o!({ "incr": "{% increment %}" })),
    );
}

#[test]
#[should_panic] // `decrement` without a variable is not supported yet
fn test_decrement_is_isolated_between_renders() {
    assert_template_result!(
        "-1-2-1",
        "{% decrement %}{% decrement %}{% render 'decr' %}",
        o!({}),
        liquid(o!({ "decr": "{% decrement %}" })),
    );
}

#[test]
#[should_panic] // We don't fail on `include` from within `render`
fn test_includes_will_not_render_inside_render_tag() {
    assert_template_result!(
        "Liquid error (test_include line 1): include usage is not allowed in this context",
        "{% render 'test_include' %}",
        o!({}),
        liquid(o!({
          "foo": "bar",
          "test_include": "{% include 'foo' %}",
        })),
    );
}

#[test]
#[should_panic] // We don't fail on `include` from within `render`
fn test_includes_will_not_render_inside_nested_sibling_tags() {
    assert_template_result!(
        "Liquid error (test_include line 1): include usage is not allowed in this context",
        "{% render 'nested_render_with_sibling_include' %}",
        o!({}),
        liquid(o!({
          "foo": "bar",
          "nested_render_with_sibling_include": "{% render 'test_include' %}{% include 'foo' %}",
          "test_include": "{% include 'foo' %}",
        })),
    );
}

#[test]
#[should_panic] // Implicit name is not supported yet
fn test_render_tag_with() {
    assert_template_result!(
        "Product: Draft 151cm ",
        "{% render 'product' with products[0] %}",
        o!({ "products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }] }),
        liquid(o!({
          "product": "Product: {{ product.title }} ",
          "product_alias": "Product: {{ product.title }} ",
        })),
    );
}

#[test]
fn test_render_tag_with_alias() {
    assert_template_result!(
        "Product: Draft 151cm ",
        "{% render 'product_alias' with products[0] as product %}",
        o!({ "products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }] }),
        liquid(o!({
          "product": "Product: {{ product.title }} ",
          "product_alias": "Product: {{ product.title }} ",
        })),
    );
}

#[test]
fn test_render_tag_for_alias() {
    assert_template_result!(
        "Product: Draft 151cm Product: Element 155cm ",
        "{% render 'product_alias' for products as product %}",
        o!({ "products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }] }),
        liquid(o!({
          "product": "Product: {{ product.title }} ",
          "product_alias": "Product: {{ product.title }} ",
        })),
    );
}

#[test]
fn test_render_tag_for() {
    assert_template_result!(
        "Product: Draft 151cm Product: Element 155cm ",
        "{% render 'product' for products as product %}",
        o!({ "products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }] }),
        liquid(o!({
          "product": "Product: {{ product.title }} ",
          "product_alias": "Product: {{ product.title }} ",
        })),
    );
}

#[test]
fn test_render_tag_forloop() {
    assert_template_result!(
        "Product: Draft 151cm first  index:1 Product: Element 155cm  last index:2 ",
        "{% render 'product' for products as product %}",
        o!({ "products": [{ "title": "Draft 151cm" }, { "title": "Element 155cm" }] }),
        liquid(o!({
          "product": "Product: {{ product.title }} {% if forloop.first %}first{% endif %} {% if forloop.last %}last{% endif %} index:{{ forloop.index }} ",
        })),
    );
}

/*
#[test]
fn test_render_tag_for_drop() {
  assert_template_result!(
    "123",
    "{% render 'loop' for loop as value %}",
    o!({ "loop": TestEnumerable.new }),
    liquid(o!({
      "loop": "{{ value.foo }}",
    },
  )
}

#[test]
fn test_render_tag_with_drop() {
  assert_template_result!(
    "TestEnumerable",
    "{% render 'loop' with loop as value %}",
    o!({ "loop": TestEnumerable.new }),
    liquid(o!({
      "loop": "{{ value }}",
    },
  )
}

#[test]
fn test_render_tag_renders_error_with_template_name() {
    assert_template_result!(
        "Liquid error (foo line 1): standard error",
        "{% render 'foo' with errors %}",
        o!({ "errors": ErrorDrop.new }),
        liquid(o!({ "foo": "{{ foo.standard_error }}" })),
    );
}

#[test]
fn test_render_tag_renders_error_with_template_name_from_template_factory() {
  assert_template_result!(
    "Liquid error (some/path/foo line 1): standard error",
    "{% render 'foo' with errors %}",
    { "errors": ErrorDrop.new },
    liquid(o!({ "foo": "{{ foo.standard_error }}" },
    template_factory: StubTemplateFactory.new,
    render_errors: true,
  )
}
*/
