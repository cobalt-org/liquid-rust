use liquid::LiquidOptions;
use liquid::Renderable;
use liquid::Context;
use liquid::value::Value;
use liquid::parse;
use std::default::Default;
use std::fs::File;
use std::io::Read;

#[test]
pub fn run() {
    let mut text = String::new();
    File::open("./tests/simple/template.txt").unwrap().read_to_string(&mut text);
    let mut options : LiquidOptions = Default::default();
    let template = parse(&text, &mut options).unwrap();

    let mut data : Context = Default::default();
    data.values.insert("num".to_string(), Value::Num(5f32));
    data.values.insert("numTwo".to_string(), Value::Num(6f32));

    let output = template.render(&mut data);
    assert_eq!(output.unwrap(), "5 wat wot\n".to_string());
}

