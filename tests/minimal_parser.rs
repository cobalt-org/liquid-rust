use liquid_core::{
    Display_filter, Filter, FilterReflection, ParseFilter, Runtime, Value, ValueView,
};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "surround",
    description = "tests helper",
    parsed(SurroundFilter)
)]
struct SurroundFilterParser;

#[derive(Debug, Default, Display_filter)]
#[name = "surround"]
struct SurroundFilter;

impl Filter for SurroundFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        Ok(Value::scalar(format!("[{}]", input.render())))
    }
}

#[test]
fn parser_builder_stays_usable_without_stdlib() {
    let parser = liquid::ParserBuilder::new()
        .filter(SurroundFilterParser)
        .build()
        .unwrap();
    let template = parser.parse("{{ value | surround }}").unwrap();
    let globals = liquid::object!({
        "value": "core",
    });

    let rendered = template.render(&globals).unwrap();

    assert_eq!(rendered, "[core]");
}
