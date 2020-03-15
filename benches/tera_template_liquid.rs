#![feature(test)]

use serde_yaml;
extern crate test;

use liquid;

// Benches from https://github.com/djc/template-benchmarks-rs

#[bench]
pub fn big_table(b: &mut test::Bencher) {
    // 100 instead of 50 in the original benchmark to make the time bigger
    let size = 100;
    let table: Vec<_> = (0..size)
        .map(|_| {
            let inner: Vec<_> = (0..size).map(|i| i as i32).collect();
            inner
        })
        .collect();

    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse(BIG_TABLE_TEMPLATE)
        .expect("Benchmark template parsing failed");

    let data = liquid::object!({ "table": table });

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

static BIG_TABLE_TEMPLATE: &'static str = "<table>
{% for row in table %}
<tr>{% for col in row %}<td>{{ col }}</td>{% endfor %}</tr>
{% endfor %}
</table>";

#[bench]
pub fn teams(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = parser
        .parse(TEAMS_TEMPLATE)
        .expect("Benchmark template parsing failed");

    let data: liquid::Object =
        serde_yaml::from_str(TEAMS_DATA).expect("Benchmark object parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

static TEAMS_DATA: &'static str = "
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

static TEAMS_TEMPLATE: &'static str = "<html>
  <head>
    <title>{{ year }}</title>
  </head>
  <body>
    <h1>CSL {{ year }}</h1>
    <ul>
    {% for team in teams %}
      <li class=\"{% if forloop.index0 == 0 %}champion{% endif %}\">
      <b>{{team.name}}</b>: {{team.score}}
      </li>
    {% endfor %}
    </ul>
  </body>
</html>";
