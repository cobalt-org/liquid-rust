extern crate liquid;
extern crate serde_yaml;

#[test]
pub fn upcase() {
    let text = "{{ text | upcase}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: hello
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "HELLO".to_string());
}

#[test]
pub fn downcase() {
    let text = "{{ text | downcase}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: HELLO tHeRe
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "hello there".to_string());
}

#[test]
pub fn capitalize() {
    let text = "{{ text | capitalize}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: hello world
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "Hello world".to_string());
}

#[test]
#[cfg(feature = "extra-filters")]
pub fn pluralize_none() {
    let text = "{{ count | pluralize: 'one', 'many'}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
count: 0
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "many".to_string());
}

#[test]
#[cfg(feature = "extra-filters")]
pub fn pluralize_singular() {
    let text = "{{ count | pluralize: 'one', 'many'}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
count: 1
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "one".to_string());
}

#[test]
#[cfg(feature = "extra-filters")]
pub fn pluralize_plural() {
    let text = "{{ count | pluralize: 'one', 'many'}}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
count: 10
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .extra_filters()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "many".to_string());
}

#[test]
pub fn minus() {
    let text = "{{ num | minus : 2 }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
num: 4
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "2".to_string());
}

#[test]
pub fn plus() {
    let text = "{{ num | plus : 2 }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
num: 4
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "6".to_string());
}

#[test]
pub fn minus_error() {
    let text = "{{ num | minus }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
num: 4
"#,
    )
    .unwrap();
    let output = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .and_then(|template| template.render(&globals));

    assert!(output.is_err());
}

#[test]
pub fn first_numeric_array() {
    let text = "{{ nums | first }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: [12, 1]
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "12".to_string());
}

#[test]
pub fn first_string_array() {
    let text = "{{ nums | first }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: ["first", "second"]
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "first".to_string());
}

#[test]
pub fn first_char() {
    let text = "{{ nums | first }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: first
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "f".to_string());
}

#[test]
pub fn last_numeric_array() {
    let text = "{{ nums | last }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: [12, 1]
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "1".to_string());
}

#[test]
pub fn last_string_array() {
    let text = "{{ nums | last }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: ["first", "second"]
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "second".to_string());
}

#[test]
pub fn last_char() {
    let text = "{{ nums | last }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
nums: second
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "d".to_string());
}

#[test]
pub fn replace_first() {
    let text = "{{ text | replace_first: 'bar', 'foo' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "foo2bar".to_string());
}

#[test]
pub fn replace() {
    let text = "{{ text | replace: 'bar', 'foo' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "foo2foo".to_string());
}

#[test]
pub fn prepend_constant() {
    let text = "{{ text | prepend: 'fifo' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "fifobar2bar".to_string());
}

#[test]
pub fn prepend_variable() {
    let text = "{{ text | prepend: myvar }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
myvar: fifo
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "fifobar2bar".to_string());
}

#[test]
pub fn append() {
    let text = "{{ text | append: 'lifo' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: roobarb
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "roobarblifo".to_string());
}

// Got this test from example at https://shopify.github.io/liquid/filters/split/
// This is an additional test to verify the comma/space parsing is also working
// from https://github.com/cobalt-org/liquid-rust/issues/41
#[test]
pub fn split_with_comma() {
    let text = "{% assign beatles = \"John, Paul, George, Ringo\" | split: \", \" %}{% for member \
                in beatles %}{{ member }}\n{% endfor %}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "John\nPaul\nGeorge\nRingo\n".to_string());
}

// This test verifies that issue #40 is fixed (that split works)
#[test]
pub fn split_no_comma() {
    let text = "{% assign letters = \"a~b~c\" | split:\"~\" %}{% for letter in letters %}LETTER: \
                {{ letter }}\n{% endfor %}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "LETTER: a\nLETTER: b\nLETTER: c\n".to_string());
}

// Split on 1 string and re-join on another
#[test]
pub fn split_then_join() {
    let text = "{{ 'a~b~c' | split:'~' | join:', ' }}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "a, b, c".to_string());
}

// Slice single character
#[test]
pub fn slice_one() {
    let text = "{{ '0123456' | slice: 2 }}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "2".to_string());
}

// Slicing with negative start should start from end of string
#[test]
pub fn slice_negative() {
    let text = "{{ '6543210' | slice: -4, 3 }}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "321".to_string());
}

#[test]
// Slicing with overflow should fit to string size
pub fn slice_overflow() {
    let text = "{{ 'xx0123456' | slice: 2, 11 }}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "0123456".to_string());
}

#[test]
// Slicing empty string should not fail
pub fn slice_empty() {
    let text = "{{ '' | slice: 2 }}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "".to_string());
}

#[test]
// Split string, sort it then re-join
pub fn split_sort_join() {
    let text = "{{ 'zebra, octopus, giraffe, Sally Snake' | split:', ' | sort | join: ', '}}";
    let globals: liquid::value::Object = Default::default();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "Sally Snake, giraffe, octopus, zebra".to_string());
}

#[test]
pub fn modulo() {
    let text = "{{ num | modulo: 2 }}";
    let samples = [(4_f64, "0"), (3_f64, "1"), (5.1, "1.0999999999999996")];
    for t in &samples {
        let globals: liquid::value::Object =
            serde_yaml::from_str(&format!("num: {}", t.0)).unwrap();
        let template = liquid::ParserBuilder::with_liquid()
            .build()
            .unwrap()
            .parse(text)
            .unwrap();
        let output = template.render(&globals).unwrap();
        assert_eq!(output, t.1.to_string());
    }
}

#[test]
pub fn escape() {
    let text = "{{ var | escape }}";
    let samples = [
        ("abc", "abc"),
        ("", ""),
        ("<>&'\"", "&lt;&gt;&amp;&#39;&quot;"),
        ("1 < 2", "1 &lt; 2"),
        ("1 &lt; 2", "1 &amp;lt; 2"),
        ("&etc.", "&amp;etc."),
    ];
    for t in &samples {
        let mut globals = liquid::value::Object::new();
        globals.insert("var".into(), liquid::value::Value::scalar(t.0));
        let template = liquid::ParserBuilder::with_liquid()
            .build()
            .unwrap()
            .parse(text)
            .unwrap();
        let output = template.render(&globals).unwrap();
        assert_eq!(output, t.1.to_string());
    }
}

#[test]
pub fn escape_once() {
    let text = "{{ var | escape_once }}";
    let samples = [
        ("text", "text"),
        ("1 < 2 & 3", "1 &lt; 2 &amp; 3"),
        ("1 &lt; 2 &amp; 3", "1 &lt; 2 &amp; 3"),
        ("&xyz;", "&amp;xyz;"),
        ("<>&'\"", "&lt;&gt;&amp;&#39;&quot;"),
        ("&lt;&gt;&amp;&#39;&quot;", "&lt;&gt;&amp;&#39;&quot;"),
    ];
    for t in &samples {
        let mut globals = liquid::value::Object::new();
        globals.insert("var".into(), liquid::value::Value::scalar(t.0));
        let template = liquid::ParserBuilder::with_liquid()
            .build()
            .unwrap()
            .parse(text)
            .unwrap();
        let output = template.render(&globals).unwrap();
        assert_eq!(output, t.1.to_string());
    }
}

#[test]
pub fn remove_first_constant() {
    let text = "{{ text | remove_first: 'bar' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "2bar".to_string());
}

#[test]
pub fn remove_first_variable() {
    let text = "{{ text | remove_first: myvar }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
myvar: bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "2bar".to_string());
}

#[test]
pub fn remove() {
    let text = "{{ text | remove: 'bar' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: bar2bar
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "2".to_string());
}

#[test]
pub fn strip_html() {
    let text = "{{ text | strip_html }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: "<!-- <b> Comment -->Lorem <a>ipsum </b>dolor"
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "Lorem ipsum dolor".to_string());
}

#[test]
pub fn truncatewords() {
    let text = "{{ text | truncatewords: 1, '...' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: "first second third"
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "first...".to_string());
}

#[test]
pub fn default_use() {
    let text = "{{ text | default: 'bar' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: false
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "bar".to_string());
}

#[test]
pub fn default_pass() {
    let text = "{{ text | default: 'bar' }}";
    let globals: liquid::value::Object = serde_yaml::from_str(
        r#"
text: foo
"#,
    )
    .unwrap();
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .unwrap()
        .parse(text)
        .unwrap();
    let output = template.render(&globals).unwrap();
    assert_eq!(output, "foo".to_string());
}
