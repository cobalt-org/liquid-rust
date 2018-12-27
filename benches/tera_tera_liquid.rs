#![feature(test)]

extern crate serde_json;
extern crate serde_yaml;
extern crate test;

extern crate liquid;

static VARIABLE_ONLY: &'static str = "{{product.name}}";

static SIMPLE_TEMPLATE: &'static str = "
<html>
  <head>
    <title>{{ product.name }}</title>
  </head>
  <body>
    <h1>{{ product.name }} - {{ product.manufacturer | upcase }}</h1>
    <p>{{ product.summary }}</p>
    <p>£{{ product.price | times: 1.20 }} (VAT inc.)</p>
    <p>Look at reviews from your friends {{ username }}</p>
    <button>Buy!</button>
  </body>
</html>
";

static PRODUCTS: &'static str = "
username: bob
product:
  name: Moto G
  manufacturer: Motorola
  summary: A phone
  price: 100
";

#[bench]
fn bench_parsing_basic_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    b.iter(|| parser.parse(SIMPLE_TEMPLATE));
}

#[bench]
fn bench_rendering_only_variable(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    let template = parser
        .parse(VARIABLE_ONLY)
        .expect("Benchmark template parsing failed");

    let data: liquid::value::Object =
        serde_yaml::from_str(PRODUCTS).expect("Benchmark object parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

#[bench]
fn bench_rendering_basic_template(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    let template = parser
        .parse(SIMPLE_TEMPLATE)
        .expect("Benchmark template parsing failed");

    let data: liquid::value::Object =
        serde_yaml::from_str(PRODUCTS).expect("Benchmark object parsing failed");

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

fn deep_object() -> liquid::value::Object {
    let data = r#"{
                    "foo": {
                        "bar": {
                            "goo": {
                                "moo": {
                                    "cows": [
                                        {
                                            "name": "betsy",
                                            "age" : 2,
                                            "temperament": "calm"
                                        },
                                        {
                                            "name": "elsie",
                                            "age": 3,
                                            "temperament": "calm"
                                        },
                                        {
                                            "name": "veal",
                                            "age": 1,
                                            "temperament": "ornery"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                  }"#;

    let data = serde_json::from_str(data).unwrap();
    vec![("deep_object".into(), liquid::value::Value::Object(data))]
        .into_iter()
        .collect()
}

#[bench]
fn access_deep_object(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    let template = parser
        .parse("{% for cow in deep_object.foo.bar.goo.moo.cows %}{{cow.temperament}}{% endfor %}")
        .expect("Benchmark template parsing failed");

    let data = deep_object();

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}

#[bench]
fn access_deep_object_with_literal(b: &mut test::Bencher) {
    let parser = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap();
    let template = parser
        .parse(
            "
{% assign goo = deep_object.foo['bar'][\"goo\"] %}
{% for cow in goo.moo.cows %}{{cow.temperament}}
{% endfor %}",
        )
        .expect("Benchmark template parsing failed");

    let data = deep_object();

    template.render(&data).unwrap();
    b.iter(|| template.render(&data));
}
