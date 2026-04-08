use std::sync::Arc;
use std::thread;

use liquid::{ErrorMode, RenderOptions};

#[test]
fn shared_template_keeps_filter_strictness_and_error_mode_per_render() {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("{{ value | missing_filter }}")
        .unwrap();
    let template = Arc::new(template);

    let strict_template = Arc::clone(&template);
    let strict_handle = thread::spawn(move || {
        let globals = liquid::object!({ "value": "alpha" });
        let result = strict_template.render_with_options(
            &globals,
            &RenderOptions {
                strict_filters: true,
                error_mode: ErrorMode::Lenient(|_| "[strict-error]".to_owned()),
                ..RenderOptions::default()
            },
        )?;
        Ok::<_, liquid::Error>((result.output, result.errors.len()))
    });

    let lenient_template = Arc::clone(&template);
    let lenient_handle = thread::spawn(move || {
        let globals = liquid::object!({ "value": "beta" });
        let result = lenient_template.render_with_options(
            &globals,
            &RenderOptions {
                strict_filters: false,
                error_mode: ErrorMode::Strict,
                ..RenderOptions::default()
            },
        )?;
        Ok::<_, liquid::Error>((result.output, result.errors.len()))
    });

    let strict = strict_handle
        .join()
        .expect("strict thread should join")
        .unwrap();
    let lenient = lenient_handle
        .join()
        .expect("lenient thread should join")
        .unwrap();

    assert_eq!(strict, ("[strict-error]".to_owned(), 1));
    assert_eq!(lenient, ("beta".to_owned(), 0));
}

#[test]
fn shared_template_keeps_output_limits_per_render() {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("{{ value }}")
        .unwrap();
    let template = Arc::new(template);

    let limited_template = Arc::clone(&template);
    let limited_handle = thread::spawn(move || {
        let globals = liquid::object!({ "value": "abcdef" });
        limited_template.render_with_options(
            &globals,
            &RenderOptions {
                max_output_bytes: Some(3),
                error_mode: ErrorMode::Strict,
                ..RenderOptions::default()
            },
        )
    });

    let unlimited_template = Arc::clone(&template);
    let unlimited_handle = thread::spawn(move || {
        let globals = liquid::object!({ "value": "abcdef" });
        unlimited_template.render_with_options(
            &globals,
            &RenderOptions {
                max_output_bytes: None,
                error_mode: ErrorMode::Strict,
                ..RenderOptions::default()
            },
        )
    });

    let limited = limited_handle.join().expect("limited thread should join");
    let unlimited = unlimited_handle
        .join()
        .expect("unlimited thread should join")
        .unwrap();

    assert!(limited.is_err());
    assert_eq!(unlimited.output, "abcdef");
    assert!(unlimited.errors.is_empty());
}
