use liquid::*;
use snapbox::assert_data_eq;

fn compare_by_file(name: &str, globals: &Object) {
    let input_file = format!("tests/fixtures/input/{}.txt", name);
    let output_file = std::path::PathBuf::from(format!("tests/fixtures/output/{}.txt", name));

    let template = ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse_file(input_file)
        .unwrap();

    let output = template.render(globals).unwrap();

    assert_data_eq!(output, snapbox::Data::read_from(&output_file, None).raw());
}

#[test]
pub fn error_on_nonexistent_file() {
    let template = ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse_file("not-a-file.ext");
    assert!(template.is_err());
}

#[test]
pub fn example_by_file() {
    let globals = object!({
        "num": 5,
        "numTwo": 6
    });
    compare_by_file("example", &globals);
}
