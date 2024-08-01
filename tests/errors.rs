use snapbox::assert_data_eq;
use snapbox::str;

#[test]
fn test_fuzz() {
    match liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("˄{%")
    {
        Ok(_) => panic!("should fail"),
        Err(err) => assert_data_eq!(
            err.to_string(),
            str![[r#"
liquid:  --> 1:3
  |
1 | {%
  |   ^---
  |
  = expected Identifier

"#]]
        ),
    }
}
