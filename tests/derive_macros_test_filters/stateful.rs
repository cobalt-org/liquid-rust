extern crate liquid;
use liquid::compiler::{Filter, FilterArguments, FilterParameters, ParseFilter};
use liquid::derive::*;
use liquid::error::Result;
use liquid::interpreter::Context;
use liquid::interpreter::Expression;
use liquid::value::Value;

#[derive(Clone, Copy, Debug)]
enum Mood {
    Happy,
    Neutral,
    Sad,
}

#[derive(Debug, FilterParameters)]
struct TestStatefulFilterParameters {
    #[parameter(description = "", arg_type = "str")]
    arg: Expression,
}

#[derive(Clone, FilterReflection)]
#[filter(
    name = "state",
    description = "Test stateful filters.",
    parameters(TestStatefulFilterParameters)
)]
pub struct TestStatefulFilterParser {
    state: Mood,
}

impl TestStatefulFilterParser {
    pub fn new() -> Self {
        Self {
            state: Mood::Neutral,
        }
    }

    pub fn make_sad(&mut self) {
        self.state = Mood::Sad;
    }

    pub fn make_happy(&mut self) {
        self.state = Mood::Happy;
    }
}

impl ParseFilter for TestStatefulFilterParser {
    fn parse(&self, arguments: FilterArguments) -> Result<Box<Filter>> {
        let args = TestStatefulFilterParameters::from_args(arguments)?;
        let state = self.state;

        Ok(Box::new(TestStatefulFilter { args, state }))
    }
}

#[derive(Debug, Display_filter)]
#[name = "state"]
pub struct TestStatefulFilter {
    #[parameters]
    args: TestStatefulFilterParameters,
    state: Mood,
}

impl Filter for TestStatefulFilter {
    fn evaluate(&self, _input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let result = match self.state {
            Mood::Happy => format!(":-) {} :-)", args.arg),
            Mood::Sad => format!(":-( {} :-(", args.arg),
            Mood::Neutral => format!(":-| {} :-|", args.arg),
        };

        Ok(Value::scalar(result))
    }
}
