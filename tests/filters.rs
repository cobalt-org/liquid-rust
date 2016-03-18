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
pub fn downcase() {
    let text = "{{ text | downcase}}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("HELLO tHeRe".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("hello there".to_string()));
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
pub fn first() {
    let text = "{{ nums | first }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    // array of numbers
    let mut data = Context::new();
    data.set_val("nums", Value::Array(vec![Value::Num(12f32), Value::Num(1f32)]));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("12".to_string()));

    // array of strings
    let mut data = Context::new();
    data.set_val("nums", Value::Array(vec![Value::Str("first".to_owned())]));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("first".to_string()));

    let mut data = Context::new();
    data.set_val("nums", Value::Str("first".to_owned()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("f".to_string()));
}


#[test]
pub fn last() {
    let text = "{{ list | last }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    // array of numbers
    let mut data = Context::new();
    data.set_val("list", Value::Array(vec![Value::Num(12f32), Value::Num(100f32)]));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("100".to_string()));

    // array of strings
    let mut data = Context::new();
    data.set_val("list", Value::Array(vec![Value::Str("first".to_owned()), Value::Str("second".to_owned())]));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("second".to_string()));

    let mut data = Context::new();
    data.set_val("list", Value::Str("last".to_owned()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("t".to_string()));
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


#[test]
pub fn prepend() {
    let text = "{{ text | prepend: 'fifo' }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("fifobar2bar".to_string()));
}


#[test]
pub fn append() {
    let text = "{{ text | append: 'lifo' }}";
    let options : LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("roobarb".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("roobarblifo".to_string()));
}
