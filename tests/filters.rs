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
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("hello".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("HELLO".to_string()));
}

#[test]
pub fn downcase() {
    let text = "{{ text | downcase}}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("HELLO tHeRe".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("hello there".to_string()));
}

#[test]
pub fn capitalize() {
    let text = "{{ text | capitalize}}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("hello world".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("Hello World".to_string()));
}

#[test]
pub fn pluralize() {
    let text = "{{ count | pluralize: 'one', 'many'}}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("count", Value::Num(1f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("one".to_string()));


    let mut data = Context::new();
    data.set_val("count", Value::Num(0f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("many".to_string()));

    let mut data = Context::new();
    data.set_val("count", Value::Num(10f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("many".to_string()));
}

#[test]
pub fn minus() {
    let text = "{{ num | minus : 2 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("2".to_string()));
}

#[test]
pub fn plus() {
    let text = "{{ num | plus : 2 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("6".to_string()));
}

#[test]
pub fn minus_error() {
    let text = "{{ num | minus }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("num", Value::Num(4f32));

    let output = template.render(&mut data);
    assert!(output.is_err());
}

#[test]
pub fn first() {
    let text = "{{ nums | first }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    // array of numbers
    let mut data = Context::new();
    data.set_val("nums",
                 Value::Array(vec![Value::Num(12f32), Value::Num(1f32)]));

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
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    // array of numbers
    let mut data = Context::new();
    data.set_val("list",
                 Value::Array(vec![Value::Num(12f32), Value::Num(100f32)]));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("100".to_string()));

    // array of strings
    let mut data = Context::new();
    data.set_val("list",
                 Value::Array(vec![Value::Str("first".to_owned()),
                                   Value::Str("second".to_owned())]));

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
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("foo2foo".to_string()));
}

#[test]
pub fn prepend() {
    let text = "{{ text | prepend: 'fifo' }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("fifobar2bar".to_string()));

    let text = "{{ text | prepend: myvar }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));
    data.set_val("myvar", Value::Str("fifo".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("fifobar2bar".to_string()));
}

#[test]
pub fn append() {
    let text = "{{ text | append: 'lifo' }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("roobarb".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("roobarblifo".to_string()));
}

#[test]
// Got this test from example at https://shopify.github.io/liquid/filters/split/
// This is an additional test to verify the comma/space parsing is also working
// from https://github.com/cobalt-org/liquid-rust/issues/41
pub fn split_with_comma() {
    let text = "{% assign beatles = \"John, Paul, George, Ringo\" | split: \", \" %}{% for member \
                in beatles %}{{ member }}\n{% endfor %}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(),
               Some("John\nPaul\nGeorge\nRingo\n".to_string()));
}

#[test]
// This test verifies that issue https://github.com/cobalt-org/liquid-rust/issues/40 is fixed (that split works)
pub fn split_no_comma() {
    let text = "{% assign letters = \"a~b~c\" | split:\"~\" %}{% for letter in letters %}LETTER: \
                {{ letter }}\n{% endfor %}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(),
               Some("LETTER: a\nLETTER: b\nLETTER: c\n".to_string()));
}

#[test]
// Split on 1 string and re-join on another
pub fn split_then_join() {
    let text = "{{ 'a~b~c' | split:'~' | join:', ' }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("a, b, c".to_string()));
}

#[test]
// Slice single character
pub fn slice_one() {
    let text = "{{ '0123456' | slice: 2 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("2".to_string()));
}

#[test]
// Slicing with negative start should start from end of string
pub fn slice_negative() {
    let text = "{{ '6543210' | slice: -4, 3 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("321".to_string()));
}

#[test]
// Slicing with overflow should fit to string size
pub fn slice_overflow() {
    let text = "{{ 'xx0123456' | slice: 2, 10.1 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("0123456".to_string()));
}

#[test]
// Slicing empty string should not fail
pub fn slice_empty() {
    let text = "{{ '' | slice: 2 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("".to_string()));
}

#[test]
// Split string, sort it then re-join
pub fn split_sort_join() {
    let text = "{{ 'zebra, octopus, giraffe, Sally Snake' | split:', ' | sort | join: ', '}}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(),
               Some("Sally Snake, giraffe, octopus, zebra".to_string()));
}

#[test]
pub fn modulo() {
    let text = "{{ num | modulo: 2 }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();
    let mut data = Context::new();

    let samples = [(4_f32, "0"), (3_f32, "1"), (5.1, "1.0999999")];
    for t in &samples {
        data.set_val("num", Value::Num(t.0));
        assert_eq!(template.render(&mut data).unwrap(), Some(t.1.to_string()));
    }
}

#[test]
pub fn escape() {
    let input = "{{ var | escape}}";
    let options: LiquidOptions = Default::default();
    let template = parse(&input, options).unwrap();
    let mut data = Context::new();

    let samples = [("abc", "abc"), ("", ""),
                   ("<>&'\"", "&lt;&gt;&amp;&#39;&quot;"),
                   ("&etc.", "&amp;etc.")];
    for t in &samples {
        data.set_val("var", Value::Str(t.0.to_string()));
        assert_eq!(template.render(&mut data).unwrap(), Some(t.1.to_string()));
    }
}

#[test]
pub fn remove_first() {
    let text = "{{ text | remove_first: 'bar' }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("2bar".to_string()));

    let text = "{{ text | remove_first: myvar }}";
    let options: LiquidOptions = Default::default();
    let template = parse(&text, options).unwrap();

    let mut data = Context::new();
    data.set_val("text", Value::Str("bar2bar".to_string()));
    data.set_val("myvar", Value::Str("bar".to_string()));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), Some("2bar".to_string()));
}

