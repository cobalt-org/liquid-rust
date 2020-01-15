use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Value;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};

#[derive(Debug, FilterParameters)]
struct TestKeywordFilterParameters {
    #[parameter(
        description = "Optional keyword argument.",
        arg_type = "str",
        mode = "keyword"
    )]
    optional: Option<Expression>,

    #[parameter(
        description = "Required keyword argument. Must be a boolean.",
        arg_type = "bool",
        mode = "keyword"
    )]
    required: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "kw",
    description = "Filter to test keyword arguments.",
    parameters(TestKeywordFilterParameters),
    parsed(TestKeywordFilter)
)]
pub struct TestKeywordFilterParser;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "kw"]
pub struct TestKeywordFilter {
    #[parameters]
    args: TestKeywordFilterParameters,
}

impl Filter for TestKeywordFilter {
    fn evaluate(&self, _input: &Value, context: &Context<'_>) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let required = args.required;

        let result = if let Some(optional) = args.optional {
            format!("<optional: {}; required: {}>", optional, required)
        } else {
            format!("<required: {}>", required)
        };

        Ok(Value::scalar(result))
    }
}
