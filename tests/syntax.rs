extern crate liquid;

use liquid::LiquidOptions;
use liquid::Renderable;
use liquid::Context;
use liquid::parse;
use std::default::Default;

macro_rules! compare {
    ($input:expr, $output:expr) => {
        let input = $input.replace("…", " ");
        let expected = $output.replace("…", " ");
        let options: LiquidOptions = Default::default();
        let template = parse(&input, options).unwrap();

        let mut data = Context::new();
        let output = template.render(&mut data);
        assert_eq!(output.unwrap(), Some(expected));
    }
}

#[test]
pub fn no_whitespace_control() {
    compare!(
        "
topic1
……{% assign foo = \"bar\" %}
……{% if foo %}
…………-……{{ foo }}
……{% endif %}
",
        "
topic1
……
……
…………-……bar
……
"
    );
}

#[test]
pub fn simple_whitespace_control() {
    compare!(
        "
topic1
……{% assign foo = \"bar\" -%}
……{% if foo -%}
…………-……{{- foo }}
……{%- endif %}
",
        "
topic1
……-bar
"
    );
}

#[test]
pub fn double_sided_whitespace_control() {
    compare!(
        "
topic1
……{%- assign foo = \"bar\" -%}
……-……{{- foo -}}……

",
        "
topic1-bar\
"
    );
}
