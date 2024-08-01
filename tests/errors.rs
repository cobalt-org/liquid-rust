#[test]
#[should_panic]
fn test_fuzz() {
    let _ = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("˄{%");
}
