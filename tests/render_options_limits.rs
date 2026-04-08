use liquid::partials::{EagerCompiler, InMemorySource};
use liquid::{object, ErrorMode, Parser, ParserBuilder, RenderOptions};

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

fn strict_options() -> RenderOptions {
    RenderOptions {
        error_mode: ErrorMode::Strict,
        ..RenderOptions::default()
    }
}

#[test]
fn output_limit_is_enforced_mid_write() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("{{ text }}").unwrap();
    let options = RenderOptions {
        max_output_bytes: Some(5),
        ..strict_options()
    };
    let mut writer = Vec::new();

    let error = template
        .render_to_with_options(&mut writer, &object!({ "text": "abcdefgh" }), &options)
        .unwrap_err();

    assert_eq!(String::from_utf8(writer).unwrap(), "abcde");
    assert!(error.to_string().contains("Output limit exceeded"));
}

#[test]
fn render_op_limit_is_enforced_across_loop_iterations() {
    let parser = ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse("{% for i in (1..3) %}{{ i }}{% endfor %}")
        .unwrap();
    let options = RenderOptions {
        max_render_ops: Some(3),
        ..strict_options()
    };

    let error = template
        .render_with_options(&object!({}), &options)
        .unwrap_err();

    assert!(error.to_string().contains("Render limit exceeded"));
}

#[test]
fn assign_byte_limit_is_cumulative_across_nested_renders() {
    let parser = parser_with_partials(&[("snippet", "{% assign foo = 'abcd' %}")]);
    let template = parser
        .parse("{% render 'snippet' %}{% render 'snippet' %}")
        .unwrap();
    let options = RenderOptions {
        max_assign_bytes: Some(6),
        ..strict_options()
    };

    let error = template
        .render_with_options(&object!({}), &options)
        .unwrap_err();

    assert!(error.to_string().contains("Assign limit exceeded"));
}

#[test]
fn depth_limit_is_enforced_for_nested_partial_boundaries() {
    let parser = parser_with_partials(&[
        ("first", "{% render 'second' %}"),
        ("second", "{% render 'third' %}"),
        ("third", "done"),
    ]);
    let template = parser.parse("{% render 'first' %}").unwrap();
    let options = RenderOptions {
        max_depth: Some(2),
        ..strict_options()
    };

    let error = template
        .render_with_options(&object!({}), &options)
        .unwrap_err();

    assert!(error.to_string().contains("Depth limit exceeded"));
}
