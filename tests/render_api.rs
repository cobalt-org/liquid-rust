use liquid_core::{
    Display_filter, Filter, FilterReflection, ParseFilter, Runtime, Value, ValueView,
};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(name = "shouty", description = "tests helper", parsed(ShoutyFilter))]
struct ShoutyFilterParser;

#[derive(Debug, Default, Display_filter)]
#[name = "shouty"]
struct ShoutyFilter;

impl Filter for ShoutyFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        Ok(Value::scalar(format!(
            "{}!",
            input.render().to_string().to_uppercase()
        )))
    }
}

#[test]
fn default_rendering_supports_rust_registered_filters() {
    let parser = liquid::ParserBuilder::with_stdlib()
        .filter(ShoutyFilterParser)
        .build()
        .unwrap();
    let template = parser.parse("{{ greeting | shouty }}").unwrap();
    let globals = liquid::object!({
        "greeting": "hello",
    });

    let rendered = template.render(&globals).unwrap();

    assert_eq!(rendered, "HELLO!");
}
