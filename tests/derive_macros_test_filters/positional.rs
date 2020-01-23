use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

#[derive(Debug, FilterParameters)]
struct TestPositionalFilterParameters {
    #[parameter(description = "First positional argument.")]
    pos1: Expression,

    #[parameter(
        description = "Second positional argument. Must be an integer.",
        arg_type = "integer"
    )]
    pos2: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "pos",
    description = "Filter to test positional arguments.",
    parameters(TestPositionalFilterParameters),
    parsed(TestPositionalFilter)
)]
pub struct TestPositionalFilterParser;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "pos"]
pub struct TestPositionalFilter {
    #[parameters]
    args: TestPositionalFilterParameters,
}

impl Filter for TestPositionalFilter {
    fn evaluate(&self, _input: &dyn ValueView, context: &Context<'_>) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let pos1 = args.pos1.to_kstr();
        let result = if let Some(pos2) = args.pos2 {
            format!("<pos1: {}; pos2: {}>", pos1, pos2)
        } else {
            format!("<pos1: {}>", pos1)
        };

        Ok(Value::scalar(result))
    }
}
