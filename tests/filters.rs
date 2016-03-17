extern crate liquid;

use liquid::LiquidOptions;
use liquid::Renderable;
use liquid::Context;
use liquid::Value;
use liquid::parse;
use std::default::Default;

#[test]
pub fn upcase() {
    let text = "{{ text | upcase}}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("hello".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("HELLO".to_string()));
}


#[test]
pub fn capitalize() {
    let text = "{{ text | capitalize}}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("hello world".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("Hello World".to_string()));
}

#[test]
pub fn minus() {
    let text = "{{ num | minus : 2 }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("2".to_string()));
}


#[test]
pub fn plus() {
    let text = "{{ num | plus : 2 }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("6".to_string()));
}

#[test]
pub fn minus_error() {
    let text = "{{ num | minus }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert!(output.is_err());
}

#[test]
pub fn replace() {
    let text = "{{ text | replace: 'bar', 'foo' }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("foo2foo".to_string()));
}
