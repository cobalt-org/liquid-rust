#![feature(test)]

use serde_yaml;
extern crate test;

use liquid;

// Mirrors handlebars' benchmark
static ITERATE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if forloop.index0 == 0 %}champion{% endif %}\">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {% endfor %}
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
fn parse_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    b.iter(|| parser.parse(ITERATE));
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse(ITERATE)
        .expect("Benchmark template parsing failed");

    let data: liquid::Object =
        serde_yaml::from_str(ITERATE_OBJECT).expect("Benchmark object parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

static LOOP: &'static str = "BEFORE\n{% for this in real%}{{this}}{%endfor%}AFTER";

#[bench]
fn large_loop_helper(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse(LOOP)
        .expect("Benchmark template parsing failed");

    let data_wrapper: Vec<_> = (1..1000).map(|i| format!("n={}", i)).collect();
    let row_wrapper = liquid::object!({
        "real": data_wrapper.clone(),
        "dummy": data_wrapper.clone(),
    });

    template.render(&row_wrapper).unwrap();
    b.iter(|| template.render(&row_wrapper));
}
