#[macro_use]
extern crate difference;
extern crate liquid;
extern crate serde_yaml;

use liquid::*;
use std::fs::File;
use std::io::Read;

fn compare_by_file(name: &str, globals: &Object) {
    let input_file = format!("tests/fixtures/input/{}.txt", name);
    let output_file = format!("tests/fixtures/output/{}.txt", name);

    let template = ParserBuilder::with_liquid()
        .extra_filters()
        .include_source(Box::new(compiler::FilesystemInclude::new(".")))
        .build()
        .parse_file(input_file)
        .unwrap();

    let output = template.render(globals).unwrap();

    let mut comp = String::new();
    File::open(output_file)
        .unwrap()
        .read_to_string(&mut comp)
        .unwrap();

    assert_diff!(&comp, &output, " ", 0);
}

#[test]
pub fn chained_filters() {
    let globals: Object = serde_yaml::from_str(
        r#"
foo: foofoo
"#,
    ).unwrap();
    compare_by_file("chained_filters", &globals);
}

#[test]
pub fn example() {
    let globals: Object = serde_yaml::from_str(
        r#"
num: 5
numTwo: 6
"#,
    ).unwrap();
    compare_by_file("example", &globals);
}

#[test]
pub fn include() {
    let mut globals: liquid::Object = Default::default();
    globals.insert("num".to_owned(), Value::scalar(5f32));
    globals.insert("numTwo".to_owned(), Value::scalar(10f32));
    compare_by_file("include", &globals);
}

#[test]
pub fn include_with_context() {
    let globals: Object = serde_yaml::from_str(
        r#"
content: "hello, world!"
"#,
    ).unwrap();
    compare_by_file("include_with_context", &globals);
}
