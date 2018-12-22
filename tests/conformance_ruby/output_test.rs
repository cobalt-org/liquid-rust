use liquid;
use liquid::compiler::FilterResult;
use liquid::value::Value;

fn make_funny(_input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar("LOL"))
}

fn cite_funny(input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar(format!("LOL: {}", input.render())))
}

fn add_smiley(input: &Value, args: &[Value]) -> FilterResult {
    let smiley = args
        .get(0)
        .map(|s| s.to_str().into_owned())
        .unwrap_or_else(|| ":-)".to_owned());
    Ok(Value::scalar(format!("{} {}", input.render(), smiley)))
}

fn add_tag(input: &Value, args: &[Value]) -> FilterResult {
    let tag = args
        .get(0)
        .map(|s| s.to_str().into_owned())
        .unwrap_or_else(|| "p".to_owned());
    let id = args
        .get(1)
        .map(|s| s.to_str().into_owned())
        .unwrap_or_else(|| "foo".to_owned());
    Ok(Value::scalar(format!(
        r#"<{} id="{}">{}</{}>"#,
        tag,
        id,
        input.render(),
        tag
    )))
}

fn paragraph(input: &Value, _args: &[Value]) -> FilterResult {
    Ok(Value::scalar(format!("<p>{}</p>", input.render())))
}

fn link_to(input: &Value, args: &[Value]) -> FilterResult {
    let name = input;
    let url = args
        .get(0)
        .map(|s| s.to_str().into_owned())
        .unwrap_or_else(|| ":-)".to_owned());
    Ok(Value::scalar(format!(
        r#"<a href="{}">{}</a>"#,
        url,
        name.render()
    )))
}

fn liquid() -> liquid::Parser {
    liquid::ParserBuilder::new()
        .filter("make_funny", make_funny as liquid::compiler::FnFilterValue)
        .filter("cite_funny", cite_funny as liquid::compiler::FnFilterValue)
        .filter("add_smiley", add_smiley as liquid::compiler::FnFilterValue)
        .filter("add_tag", add_tag as liquid::compiler::FnFilterValue)
        .filter("paragraph", paragraph as liquid::compiler::FnFilterValue)
        .filter("link_to", link_to as liquid::compiler::FnFilterValue)
        .build()
        .unwrap()
}

fn assigns() -> liquid::value::Value {
    v!({
      "best_cars": "bmw",
      "car": { "bmw": "good", "gm": "bad" }
    })
}

#[test]
fn test_variable() {
    let text = " {{best_cars}} ";

    let expected = " bmw ";
    assert_template_result!(expected, text, assigns());
}

#[test]
fn test_variable_traversing_with_two_brackets() {
    let text = "{{ site.data.menu[include.menu][include.locale] }}";
    assert_template_result!(
        "it works!",
        text,
        v!({
          "site": { "data": { "menu": { "foo": { "bar": "it works!" } } } },
          "include": { "menu": "foo", "locale": "bar" }
        })
    );
}

#[test]
fn test_variable_traversing() {
    let text = " {{car.bmw}} {{car.gm}} {{car.bmw}} ";

    let expected = " good bad good ";
    assert_template_result!(expected, text, assigns());
}

#[test]
fn test_variable_piping() {
    let text = " {{ car.gm | make_funny }} ";
    let expected = " LOL ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_variable_piping_with_input() {
    let text = " {{ car.gm | cite_funny }} ";
    let expected = " LOL: bad ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_variable_piping_with_args() {
    let text = r#" {{ car.gm | add_smiley : ":-(" }} "#;
    let expected = " bad :-( ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_variable_piping_with_no_args() {
    let text = " {{ car.gm | add_smiley }} ";
    let expected = " bad :-) ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_multiple_variable_piping_with_args() {
    let text = r#" {{ car.gm | add_smiley : ":-(" | add_smiley : ":-("}} "#;
    let expected = " bad :-( :-( ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_variable_piping_with_multiple_args() {
    let text = r#" {{ car.gm | add_tag : "span", "bar"}} "#;
    let expected = r#" <span id="bar">bad</span> "#;

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_variable_piping_with_variable_args() {
    let text = r#" {{ car.gm | add_tag : "span", car.bmw}} "#;
    let expected = r#" <span id="good">bad</span> "#;

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_multiple_pipings() {
    let text = " {{ best_cars | cite_funny | paragraph }} ";
    let expected = " <p>LOL: bmw</p> ";

    assert_template_result!(expected, text, assigns(), liquid());
}

#[test]
fn test_link_to() {
    let text = r#" {{ "Typo" | link_to: "http://typo.leetsoft.com" }} "#;
    let expected = r#" <a href="http://typo.leetsoft.com">Typo</a> "#;

    assert_template_result!(expected, text, assigns(), liquid());
}
