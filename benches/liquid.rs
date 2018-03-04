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

    let data = liquid::Object::new();

    b.iter(|| template.render(&data));
}

// Mirrors tera's VARIABLE_ONLY benchmark
static VARIABLE_ONLY: &'static str = "{{product.name}}";
static VARIABLE_ONLY_OBJECT: &'static str = "
username: bob
product:
  - name: Moto G
  - manufacturer: Motorola
  - summary: A phone
  - price: 100
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

    let data: liquid::Object =
        serde_yaml::from_str(VARIABLE_ONLY_OBJECT).expect("Benchmark object parsing failed");

    b.iter(|| template.render(&data));
}

// Mirrors handlebars' benchmark
static ITERATE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"champion\">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {{/each}}
    </ul>
  </body>
</html>";
static ITERATE_OBJECT: &'static str = "
year: 2015
teams:
  - name: Jiangsu
    score: 43
  - name: Beijing
    score: 27
  - name: Guangzhou
    score: 22
  - name: Shandong
    score: 12
";

#[bench]
fn bench_parse_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    b.iter(|| parser.parse(ITERATE));
}

#[bench]
fn bench_render_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    let template = parser
        .parse(ITERATE)
        .expect("Benchmark template parsing failed");

    let data: liquid::Object =
        serde_yaml::from_str(ITERATE_OBJECT).expect("Benchmark object parsing failed");

    b.iter(|| template.render(&data));
}
