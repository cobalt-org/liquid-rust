#[macro_use]
extern crate difference;
extern crate liquid;
extern crate serde_yaml;

use std::fs::File;
use std::io::Read;

use liquid::*;

fn compare_by_file(name: &str, globals: &value::Object) {
    let input_file = format!("tests/fixtures/input/{}.txt", name);
    let output_file = format!("tests/fixtures/output/{}.txt", name);

    let mut partials = liquid::Partials::empty();
    partials.add("tests/fixtures/input/example.txt", r#"{{'whooo' | size}}{%comment%}What happens{%endcomment%} {%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}
"#);
    partials.add(
        "tests/fixtures/input/include_with_val.txt",
        r#"{{content}}
"#,
    );

    let template = ParserBuilder::with_liquid()
        .extra_filters()
        .partials(partials)
        .build()
        .unwrap()
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
    let globals: value::Object = serde_yaml::from_str(
        r#"
foo: foofoo
"#,
    )
    .unwrap();
    compare_by_file("chained_filters", &globals);
}

#[test]
pub fn example() {
    let globals: value::Object = serde_yaml::from_str(
        r#"
num: 5
numTwo: 6
"#,
    )
    .unwrap();
    compare_by_file("example", &globals);
}

#[test]
pub fn include() {
    let mut globals: value::Object = Default::default();
    globals.insert("num".into(), value::Value::scalar(5f64));
    globals.insert("numTwo".into(), value::Value::scalar(10f64));
    compare_by_file("include", &globals);
}

#[test]
pub fn include_with_context() {
    let globals: value::Object = serde_yaml::from_str(
        r#"
content: "hello, world!"
"#,
    )
    .unwrap();
    compare_by_file("include_with_context", &globals);
}
