use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

#[derive(Debug, FilterParameters)]
struct TestKeywordGroupFilterParameters {
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

    #[parameter(
        description = "Keyword group that contains all keyword parameters.",
        mode = "keyword_group"
    )]
    all_keywords: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "kwg",
    description = "Filter to test keyword group arguments.",
    parameters(TestKeywordGroupFilterParameters),
    parsed(TestKeywordGroupFilter)
)]
pub struct TestKeywordGroupFilterParser;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "kwg"]
pub struct TestKeywordGroupFilter {
    #[parameters]
    args: TestKeywordGroupFilterParameters,
}

impl Filter for TestKeywordGroupFilter {
    fn evaluate(&self, _input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let required = args.required;

        let binding = args.all_keywords.to_value();
        let keyword_group = binding.as_object().unwrap();
        let required_from_keyword_group = keyword_group
            .get("required")
            .unwrap()
            .as_scalar()
            .unwrap()
            .to_bool()
            .unwrap();

        let result = if let Some(optional) = args.optional {
            let optional_from_keyword_group = keyword_group.get("optional").unwrap().to_kstr();
            format!(
                "<optional: {}; required: {}; all_keywords.required: {}; all_keywords.optional: {}>",
                optional,
                required,
                required_from_keyword_group,
                optional_from_keyword_group
            )
        } else {
            format!(
                "<required: {}; all_keywords.required: {}>",
                required, required_from_keyword_group
            )
        };

        Ok(Value::scalar(result))
    }
}
