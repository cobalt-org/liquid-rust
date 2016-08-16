extern crate difference;
extern crate liquid;

use std::fs::File;
use std::io::Read;
use liquid::*;

// README: The compare_by_file and following tests are almost line for line carbon-copies of the
// tests in `fixutres.rs`. This might be overkill but keep that in mind when making changes to
// fixtures that might necessitate changes to the parse_file method tested here.

fn compare_by_file(name: &str, context: &mut Context) {
    let input_file = format!("tests/fixtures/input/{}.txt", name);
    let output_file = format!("tests/fixtures/output/{}.txt", name);

    let options: LiquidOptions = Default::default();
    let template = parse_file(&input_file, options).unwrap();

    let output = template.render(context).unwrap();

    let mut comp = String::new();
    File::open(output_file).unwrap().read_to_string(&mut comp).unwrap();

    difference::assert_diff(&comp, &output.unwrap(), " ", 0);
}

#[test]
pub fn error_on_nonexistent_file() {
    // Assert that parsing a bogus file with default options Results in an error.
    let options = Default::default();
    let template = parse_file("not-a-file.ext", options);
    assert_eq!(template.is_ok(), false);
}

#[test]
pub fn chained_filters_by_file() {
    let mut context = Context::new();
    context.set_val("foo", Value::Str("foofoo".to_owned()));
    compare_by_file("chained_filters", &mut context)
}

#[test]
pub fn example_by_file() {
    let mut context = Context::new();
    context.set_val("num", Value::Num(5f32));
    context.set_val("numTwo", Value::Num(6f32));
    compare_by_file("example", &mut context)
}

#[test]
pub fn include_by_file() {
    let mut context = Context::new();
    compare_by_file("include", &mut context);
}

#[test]
pub fn include_with_context_by_file() {
    let mut context = Context::new();
    context.set_val("content", Value::Str("hello, world!".to_owned()));

    compare_by_file("include_with_context", &mut context);
}
