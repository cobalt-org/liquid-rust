#[cfg(not(feature = "conformance-harness"))]
#[test]
fn default_build_uses_public_rust_render_path() {
    assert!(!cfg!(feature = "conformance-harness"));

    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser.parse("Hello {{ name | upcase }}!").unwrap();
    let globals = liquid::object!({
        "name": "liquid",
    });

    let rendered = template.render(&globals).unwrap();

    assert_eq!(rendered, "Hello LIQUID!");
}

#[cfg(feature = "conformance-harness")]
#[test]
fn workspace_validation_allows_conformance_harness_feature_unification() {
    assert!(cfg!(feature = "conformance-harness"));
}
