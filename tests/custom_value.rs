use liquid::model::{DisplayCow, KStringCow, State, Value};
use liquid::{ObjectView, ValueView};
use liquid_core::model::CustomValue;
use std::sync::Arc;

/// Renders as its url, answers `.url` as an object. The two disagree, so a
/// `Value` that has flattened it is distinguishable from one that kept it.
#[derive(Debug)]
struct Link(String);

impl ValueView for Link {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }
    fn render(&self) -> DisplayCow<'_> {
        DisplayCow::Borrowed(&self.0)
    }
    fn source(&self) -> DisplayCow<'_> {
        DisplayCow::Borrowed(&self.0)
    }
    fn type_name(&self) -> &'static str {
        "object"
    }
    fn query_state(&self, state: State) -> bool {
        matches!(state, State::Truthy)
    }
    fn to_kstr(&self) -> KStringCow<'_> {
        KStringCow::from_ref(&self.0)
    }
    fn to_value(&self) -> Value {
        liquid::object!({ "url": self.0.clone() }).into()
    }
    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl ObjectView for Link {
    fn as_value(&self) -> &dyn ValueView {
        self
    }
    fn size(&self) -> i64 {
        1
    }
    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = KStringCow<'k>> + 'k> {
        Box::new(std::iter::once(KStringCow::from_static("url")))
    }
    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        Box::new(std::iter::once(&self.0 as &dyn ValueView))
    }
    fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        Box::new(std::iter::once((
            KStringCow::from_static("url"),
            &self.0 as &dyn ValueView,
        )))
    }
    fn contains_key(&self, index: &str) -> bool {
        index == "url"
    }
    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        (index == "url").then_some(&self.0 as &dyn ValueView)
    }
}

impl CustomValue for Link {}

const URL: &str = "https://example.com/a.jpg";

fn link() -> Value {
    Value::Custom(Arc::new(Link(URL.to_owned())))
}

/// `liquid::object!` builds through serde, which flattens a custom value into
/// its owned form; the context has to carry the `Value` itself.
fn object(entries: Vec<(&str, Value)>) -> liquid::Object {
    entries
        .into_iter()
        .map(|(k, v)| (liquid::model::KString::from_ref(k), v))
        .collect()
}

fn render(template: &str, globals: &liquid::Object) -> String {
    liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(template)
        .unwrap()
        .render(globals)
        .unwrap()
}

#[test]
fn a_custom_value_renders_itself_and_keeps_its_keys() {
    let globals = object(vec![("photo", link())]);
    assert_eq!(render("{{ photo }}", &globals), URL);
    assert_eq!(render("{{ photo.url }}", &globals), URL);
}

/// The points a borrow cannot cross: a loop item and an assigned variable are
/// owned `Value`s by the time anything renders them.
#[test]
fn a_custom_value_survives_assign_and_loops() {
    let globals = object(vec![(
        "posts",
        Value::Array(vec![Value::Object(object(vec![("image", link())]))]),
    )]);
    assert_eq!(
        render("{% for p in posts %}{{ p.image }}{% endfor %}", &globals),
        URL
    );
    assert_eq!(
        render(
            "{% for p in posts %}{{ p.image.url }}{% endfor %}",
            &globals
        ),
        URL
    );
    assert_eq!(
        render(
            "{% assign p = posts[0].image %}{{ p }}|{{ p.url }}",
            &globals
        ),
        format!("{URL}|{URL}")
    );
    assert_eq!(
        render(
            "{% capture c %}{{ posts[0].image }}{% endcapture %}{{ c }}",
            &globals
        ),
        URL
    );
}

/// Read as data it is still whatever `to_value` says, so json consumers are
/// unaffected.
#[test]
fn a_custom_value_serializes_as_its_owned_form() {
    let json = serde_json::to_string(&link()).unwrap();
    assert_eq!(json, format!(r#"{{"url":"{URL}"}}"#));
}
