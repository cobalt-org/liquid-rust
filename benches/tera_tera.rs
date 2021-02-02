#[macro_use]
extern crate serde_derive;
use serde_json;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tera::{escape_html, Context, Template, Tera, Value};

static VARIABLE_ONLY: &'static str = "{{product.name}}";

static SIMPLE_TEMPLATE: &'static str = "
<html>
  <head>
    <title>{{ product.name }}</title>
  </head>
  <body>
    <h1>{{ product.name }} - {{ product.manufacturer | upper }}</h1>
    <p>{{ product.summary }}</p>
    <p>£{{ product.price * 1.20 }} (VAT inc.)</p>
    <p>Look at reviews from your friends {{ username }}</p>
    <button>Buy!</button>
  </body>
</html>
";

static PARENT_TEMPLATE: &'static str = "
<html>
  <head>
    <title>{% block title %}Hello{% endblock title%}</title>
  </head>
  <body>
    {% block body %}{% endblock body %}
  </body>
</html>
";

static MACRO_TEMPLATE: &'static str = "
{% macro render_product(product) %}
    <h1>{{ product.name }} - {{ product.manufacturer | upper }}</h1>
    <p>{{ product.summary }}</p>
    <p>£{{ product.price * 1.20 }} (VAT inc.)</p>
    <button>Buy!</button>
{% endmacro render_product %}
";

static CHILD_TEMPLATE: &'static str = r#"{% extends "parent.html" %}
{% block title %}{{ super() }} - {{ username | lower }}{% endblock title %}

{% block body %}body{% endblock body %}
"#;

static CHILD_TEMPLATE_WITH_MACRO: &'static str = r#"{% extends "parent.html" %}
{% import "macros.html" as macros %}

{% block title %}{{ super() }} - {{ username | lower }}{% endblock title %}

{% block body %}
{{ macros::render_product(product=product) }}
{% endblock body %}
"#;

static USE_MACRO_TEMPLATE: &'static str = r#"
{% import "macros.html" as macros %}
{{ macros::render_product(product=product) }}
"#;

#[derive(Debug, Serialize)]
struct Product {
    name: String,
    manufacturer: String,
    price: i32,
    summary: String,
}
impl Product {
    pub fn new() -> Product {
        Product {
            name: "Moto G".to_owned(),
            manufacturer: "Motorala".to_owned(),
            summary: "A phone".to_owned(),
            price: 100,
        }
    }
}

fn bench_parsing_basic_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        b.iter(|| Template::new("bench", None, SIMPLE_TEMPLATE));
    });
    group.finish();
}

fn bench_parsing_with_inheritance_and_macros(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        b.iter(|| {
            tera.add_raw_templates(vec![
                ("parent.html", PARENT_TEMPLATE),
                ("child.html", CHILD_TEMPLATE),
                ("macros.html", MACRO_TEMPLATE),
            ])
        });
    });
    group.finish();
}

fn bench_rendering_only_variable(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_template("test.html", VARIABLE_ONLY).unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("test.html", &context));
    });
    group.finish();
}

fn bench_rendering_basic_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_template("bench.html", SIMPLE_TEMPLATE)
            .unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("bench.html", &context));
    });
    group.finish();
}

fn bench_rendering_only_parent(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("parent.html", PARENT_TEMPLATE)])
            .unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("parent.html", &context));
    });
    group.finish();
}

fn bench_rendering_only_macro_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("hey.html", USE_MACRO_TEMPLATE),
            ("macros.html", MACRO_TEMPLATE),
        ])
        .unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("hey.html", &context));
    });
    group.finish();
}

fn bench_rendering_only_inheritance(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("parent.html", PARENT_TEMPLATE),
            ("child.html", CHILD_TEMPLATE),
        ])
        .unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("child.html", &context));
    });
    group.finish();
}

fn bench_rendering_inheritance_and_macros(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("parent.html", PARENT_TEMPLATE),
            ("child.html", CHILD_TEMPLATE_WITH_MACRO),
            ("macros.html", MACRO_TEMPLATE),
        ])
        .unwrap();
        let mut context = Context::new();
        context.insert("product", &Product::new());
        context.insert("username", &"bob");

        b.iter(|| tera.render("child.html", &context));
    });
    group.finish();
}

fn bench_build_inheritance_chains(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("parent.html", PARENT_TEMPLATE),
            ("child.html", CHILD_TEMPLATE_WITH_MACRO),
            ("macros.html", MACRO_TEMPLATE),
        ])
        .unwrap();
        b.iter(|| tera.build_inheritance_chains());
    });
    group.finish();
}

fn bench_escape_html(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        b.iter(|| escape_html(r#"Hello word <script></script>"#));
    });
    group.finish();
}

fn bench_huge_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        #[derive(Serialize)]
        struct DataWrapper {
            v: String,
        }

        #[derive(Serialize)]
        struct RowWrapper {
            real: Vec<DataWrapper>,
            dummy: Vec<DataWrapper>,
        }
        let real: Vec<DataWrapper> = (1..1000)
            .into_iter()
            .map(|i| DataWrapper {
                v: format!("n={}", i),
            })
            .collect();
        let dummy: Vec<DataWrapper> = (1..1000)
            .into_iter()
            .map(|i| DataWrapper {
                v: format!("n={}", i),
            })
            .collect();
        let rows = RowWrapper { real, dummy };

        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("huge.html", "{% for v in rows %}{{v}}{% endfor %}")])
            .unwrap();
        let mut context = Context::new();
        context.insert("rows", &rows);

        b.iter(|| tera.render("huge.html", &context));
    });
    group.finish();
}

fn deep_object() -> Value {
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

    serde_json::from_str(data).unwrap()
}

fn bench_access_deep_object(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "deep_object.html",
            "{% for cow in deep_object.foo.bar.goo.moo.cows %}{{cow.temperament}}{% endfor %}",
        )])
        .unwrap();
        let mut context = Context::new();
        println!("{:?}", deep_object());
        context.insert("deep_object", &deep_object());
        assert!(tera
            .render("deep_object.html", &context)
            .unwrap()
            .contains("ornery"));

        b.iter(|| tera.render("deep_object.html", &context));
    });
    group.finish();
}

fn bench_access_deep_object_with_literal(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixtures");
    group.bench_function(BenchmarkId::new("render", "tera"), |b| {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![(
            "deep_object.html",
            "
{% set goo = deep_object.foo['bar'][\"goo\"] %}
{% for cow in goo.moo.cows %}{{cow.temperament}}
{% endfor %}",
        )])
        .unwrap();
        let mut context = Context::new();
        context.insert("deep_object", &deep_object());
        assert!(tera
            .render("deep_object.html", &context)
            .unwrap()
            .contains("ornery"));

        b.iter(|| tera.render("deep_object.html", &context));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parsing_basic_template,
    bench_parsing_with_inheritance_and_macros,
    bench_rendering_only_variable,
    bench_rendering_basic_template,
    bench_rendering_only_parent,
    bench_rendering_only_macro_call,
    bench_rendering_only_inheritance,
    bench_rendering_inheritance_and_macros,
    bench_build_inheritance_chains,
    bench_escape_html,
    bench_huge_loop,
    bench_access_deep_object,
    bench_access_deep_object_with_literal,
);
criterion_main!(benches);
