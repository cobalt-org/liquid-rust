use liquid::partials::{EagerCompiler, InMemorySource};
use liquid::{object, Error, ErrorMode, Parser, ParserBuilder, RenderOptions};

fn parser_with_partials(partials: &[(&str, &str)]) -> Parser {
    let mut source = InMemorySource::new();
    for (name, template) in partials {
        source.add(*name, *template);
    }

    ParserBuilder::with_stdlib()
        .partials(EagerCompiler::new(source))
        .build()
        .unwrap()
}

fn bracket_error(error: &Error) -> String {
    let rendered = error.to_string();
    let first_line = rendered.lines().next().unwrap_or_default();
    format!("[[{first_line}]]")
}

#[test]
fn strict_mode_aborts_on_first_render_error() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("{% render 'missing' %}").unwrap();
    let options = RenderOptions {
        error_mode: ErrorMode::Strict,
        ..RenderOptions::default()
    };

    let error = template
        .render_with_options(&object!({}), &options)
        .unwrap_err();

    assert!(error.to_string().contains("Unknown partial-template"));
}

#[test]
fn lenient_mode_inlines_and_collects_render_errors() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("before {% render 'missing' %} after").unwrap();
    let options = RenderOptions {
        error_mode: ErrorMode::Lenient(bracket_error),
        ..RenderOptions::default()
    };

    let output = template
        .render_with_options(&object!({}), &options)
        .unwrap();

    assert_eq!(
        output.output,
        "before [[liquid: Unknown partial-template]] after"
    );
    assert_eq!(output.errors.len(), 1);
    assert!(output.errors[0]
        .to_string()
        .contains("Unknown partial-template"));
}

#[test]
fn missing_variables_are_nil_by_default_even_in_isolated_scopes() {
    let parser = parser_with_partials(&[("snippet", "{{ missing }}")]);
    let template = parser.parse("{% render 'snippet' %}").unwrap();

    assert_eq!(template.render(&object!({})).unwrap(), "");

    let strict = RenderOptions {
        strict_variables: true,
        error_mode: ErrorMode::Strict,
        ..RenderOptions::default()
    };
    let error = template
        .render_with_options(&object!({}), &strict)
        .unwrap_err();
    assert!(error.to_string().contains("Unknown variable"));
}

#[test]
fn missing_partial_arguments_default_to_nil_in_render_and_include() {
    let parser = parser_with_partials(&[("snippet", "[{{ value }}]")]);
    let render = parser
        .parse("{% render 'snippet', value: missing %}")
        .unwrap();
    let include = parser
        .parse("{% include 'snippet', value: missing %}")
        .unwrap();

    assert_eq!(render.render(&object!({})).unwrap(), "[]");
    assert_eq!(include.render(&object!({})).unwrap(), "[]");

    let strict = RenderOptions {
        strict_variables: true,
        error_mode: ErrorMode::Strict,
        ..RenderOptions::default()
    };
    assert!(render
        .render_with_options(&object!({}), &strict)
        .unwrap_err()
        .to_string()
        .contains("Unknown variable"));
    assert!(include
        .render_with_options(&object!({}), &strict)
        .unwrap_err()
        .to_string()
        .contains("Unknown variable"));
}

#[test]
fn missing_filters_pass_input_through_by_default() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("{{ value | unknown_filter }}").unwrap();

    assert_eq!(
        template.render(&object!({ "value": "hello" })).unwrap(),
        "hello"
    );

    let strict = RenderOptions {
        strict_filters: true,
        error_mode: ErrorMode::Strict,
        ..RenderOptions::default()
    };
    let error = template
        .render_with_options(&object!({ "value": "hello" }), &strict)
        .unwrap_err();
    assert!(error.to_string().contains("Unknown filter"));
}

#[test]
fn blank_conditional_else_and_elsif_branches_do_not_force_inline_errors() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let options = RenderOptions {
        strict_filters: true,
        error_mode: ErrorMode::Lenient(bracket_error),
        ..RenderOptions::default()
    };
    let assigns = object!({ "value": "hello", "other": false });

    for source in [
        "{% if value | unknown_filter %}{% else %}{% endif %}",
        "{% if value | unknown_filter %}{% elsif other %}{% endif %}",
        "{% unless value | unknown_filter %}{% else %}{% endunless %}",
    ] {
        let template = parser.parse(source).unwrap();
        let output = template.render_with_options(&assigns, &options).unwrap();

        assert_eq!(output.output, "");
        assert!(output.errors.is_empty());
    }
}

#[test]
fn visible_conditional_else_branch_still_surfaces_condition_errors() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse("{% if value | unknown_filter %}{% else %}fallback{% endif %}")
        .unwrap();
    let options = RenderOptions {
        strict_filters: true,
        error_mode: ErrorMode::Lenient(bracket_error),
        ..RenderOptions::default()
    };

    let output = template
        .render_with_options(&object!({ "value": "hello" }), &options)
        .unwrap();

    assert_eq!(output.output, "[[liquid: Unknown filter]]");
    assert_eq!(output.errors.len(), 1);
    assert!(output.errors[0].to_string().contains("Unknown filter"));
}

#[test]
fn blank_conditional_true_branch_tags_do_not_force_inline_errors() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let options = RenderOptions {
        strict_filters: true,
        error_mode: ErrorMode::Lenient(bracket_error),
        ..RenderOptions::default()
    };
    let assigns = object!({ "value": "hello" });

    for source in [
        "{% if value | unknown_filter %}{% comment %}hidden{% endcomment %}{% endif %}",
        "{% if value | unknown_filter %}{% capture hidden %}{% endcapture %}{% endif %}",
        "{% unless value | unknown_filter %}{% comment %}hidden{% endcomment %}{% endunless %}",
    ] {
        let template = parser.parse(source).unwrap();
        let output = template.render_with_options(&assigns, &options).unwrap();

        assert_eq!(output.output, "");
        assert!(output.errors.is_empty());
    }
}
