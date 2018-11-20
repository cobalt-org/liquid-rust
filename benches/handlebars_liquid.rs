#![feature(test)]

extern crate serde_yaml;
extern crate test;

extern crate liquid;

// Mirrors handlebars' benchmark
static ITERATE: &'static str = "<html>
  <head>
    <title>{{year}}</title>
  </head>
  <body>
    <h1>CSL {{year}}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if forloop.index0 == 0 %}champion{{% endif %}\">
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
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    b.iter(|| parser.parse(ITERATE));
}

#[bench]
fn render_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    let template = parser
        .parse(ITERATE)
        .expect("Benchmark template parsing failed");

    let data: liquid::value::Object =
        serde_yaml::from_str(ITERATE_OBJECT).expect("Benchmark object parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

static LOOP: &'static str = "BEFORE\n{% for this in real%}{{this}}{%endfor%}AFTER";

#[bench]
fn large_loop_helper(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid().extra_filters().build();
    let template = parser
        .parse(LOOP)
        .expect("Benchmark template parsing failed");

    let data_wrapper = liquid::value::Value::array(
        (1..1000)
            .map(|i| format!("n={}", i))
            .map(liquid::value::Value::scalar),
    );
    let row_wrapper: liquid::value::Object = vec![
        ("real".into(), data_wrapper.clone()),
        ("dummy".into(), data_wrapper.clone()),
    ].into_iter()
    .collect();

    template.render(&row_wrapper).unwrap();
    b.iter(|| template.render(&row_wrapper));
}
