#![feature(test)]

extern crate test;

use liquid;

static TEXT_ONLY: &'static str = "Hello World";

#[bench]
fn bench_parse_text(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    b.iter(|| parser.parse(TEXT_ONLY));
}

#[bench]
fn bench_render_text(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    let template = parser
        .parse(TEXT_ONLY)
        .expect("Benchmark template parsing failed");

    let data = liquid::Object::new();

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}
