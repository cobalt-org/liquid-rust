#[macro_use]
extern crate difference;
extern crate liquid;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use liquid::*;

fn options() -> LiquidOptions {
    LiquidOptions {
        include_source: Box::new(FilesystemInclude::new(PathBuf::from("."))),
        ..Default::default()
    }
}

fn compare(name: &str, context: &mut Context) {
    let input_file = format!("tests/fixtures/input/{}.txt", name);
    let output_file = format!("tests/fixtures/output/{}.txt", name);
    let mut input = String::new();
    File::open(Path::new(&input_file))
        .unwrap()
        .read_to_string(&mut input)
        .unwrap();

    let template = parse(&input, options()).unwrap();

    let output = template.render(context).unwrap();

    let mut comp = String::new();
    File::open(output_file)
        .unwrap()
        .read_to_string(&mut comp)
        .unwrap();

    assert_diff!(&comp, &output.unwrap(), " ", 0);
}

#[test]
pub fn chained_filters() {
    let mut context = Context::new();
    context.set_val("foo", Value::Str("foofoo".to_owned()));
    compare("chained_filters", &mut context)
}

#[test]
pub fn example() {
    let mut context = Context::new();
    context.set_val("num", Value::Num(5f32));
    context.set_val("numTwo", Value::Num(6f32));
    compare("example", &mut context)
}

#[test]
pub fn include() {
    let mut context = Context::new();
    compare("include", &mut context);
}

#[test]
pub fn include_with_context() {
    let mut context = Context::new();
    context.set_val("content", Value::Str("hello, world!".to_owned()));

    compare("include_with_context", &mut context);
}
