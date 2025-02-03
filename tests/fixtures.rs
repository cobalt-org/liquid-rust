use liquid::*;
use snapbox::assert_data_eq;

pub type Partials = partials::EagerCompiler<partials::InMemorySource>;

fn compare_by_file(name: &str, globals: &Object) {
    let input_file = format!("tests/fixtures/input/{name}.txt");
    let output_file = std::path::PathBuf::from(format!("tests/fixtures/output/{name}.txt"));

    let mut partials = Partials::empty();
    partials.add("tests/fixtures/input/example.txt", r#"{{'whooo' | size}}{%comment%}What happens{%endcomment%} {%if num < numTwo%}wat{%else%}wot{%endif%} {%if num > numTwo%}wat{%else%}wot{%endif%}
"#);
    partials.add(
        "tests/fixtures/input/include_with_val.txt",
        r#"{{content}}
"#,
    );

    let template = ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file(input_file)
        .unwrap();

    let output = template.render(globals).unwrap();

    assert_data_eq!(output, snapbox::Data::read_from(&output_file, None).raw());
}

#[test]
pub fn chained_filters() {
    let globals = object!({
        "foo": "foofoo",
    });
    compare_by_file("chained_filters", &globals);
}

#[test]
pub fn example() {
    let globals = object!({
        "num": 5,
        "numTwo": 6
    });
    compare_by_file("example", &globals);
}

#[test]
pub fn include() {
    let globals = object!({
        "num": 5f64,
        "numTwo": 10f64,
    });
    compare_by_file("include", &globals);
}

#[test]
pub fn include_with_context() {
    let globals = object!({
        "content": "hello, world!",
    });
    compare_by_file("include_with_context", &globals);
}
