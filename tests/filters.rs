extern crate liquid;

use liquid::LiquidOptions;
use liquid::Renderable;
use liquid::Context;
use liquid::Value;
use liquid::parse;
use std::default::Default;

#[test]
pub fn upcase() {
    let mut text = "{{ text | upcase}}";
    let mut options : LiquidOptions = Default::default();
    let template = parse(&text, &mut options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("hello".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("HELLO".to_string()));
}
