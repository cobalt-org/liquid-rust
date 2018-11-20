#![feature(test)]

extern crate serde_yaml;
extern crate test;

extern crate liquid;

static TEXT_ONLY: &'static str = "Hello World";

#[bench]
fn bench_parse_text(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    b.iter(|| parser.parse(TEXT_ONLY));
}

#[bench]
fn bench_render_text(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    let template = parser
        .parse(TEXT_ONLY)
        .expect("Benchmark template parsing failed");

    let data = liquid::value::Object::new();

    b.iter(|| template.render(&data));
}

// Mirrors tera's VARIABLE_ONLY benchmark
static VARIABLE_ONLY: &'static str = "{{product.name}}";
static VARIABLE_ONLY_OBJECT: &'static str = "
username: bob
product:
  name: Moto G
  manufacturer: Motorola
  summary: A phone
  price: 100
";

#[bench]
fn bench_parse_variable(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    b.iter(|| parser.parse(VARIABLE_ONLY));
}

#[bench]
fn bench_render_variable(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    let template = parser
        .parse(VARIABLE_ONLY)
        .expect("Benchmark template parsing failed");

    let data: liquid::value::Object =
        serde_yaml::from_str(VARIABLE_ONLY_OBJECT).expect("Benchmark object parsing failed");

    b.iter(|| template.render(&data));
}
