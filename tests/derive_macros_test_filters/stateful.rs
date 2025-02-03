use liquid_core::parser::FilterArguments;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{Display_filter, Filter, FilterParameters, FilterReflection, ParseFilter};
use liquid_core::{Value, ValueView};

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
pub(crate) struct TestStatefulFilterParser {
    state: Mood,
}

impl TestStatefulFilterParser {
    pub(crate) fn new() -> Self {
        Self {
            state: Mood::Neutral,
        }
    }

    pub(crate) fn make_sad(&mut self) {
        self.state = Mood::Sad;
    }

    pub(crate) fn make_happy(&mut self) {
        self.state = Mood::Happy;
    }
}

impl ParseFilter for TestStatefulFilterParser {
    fn parse(&self, arguments: FilterArguments<'_>) -> Result<Box<dyn Filter>> {
        let args = TestStatefulFilterParameters::from_args(arguments)?;
        let state = self.state;

        Ok(Box::new(TestStatefulFilter { args, state }))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self
    }
}

#[derive(Debug, Display_filter)]
#[name = "state"]
pub(super) struct TestStatefulFilter {
    #[parameters]
    args: TestStatefulFilterParameters,
    state: Mood,
}

impl Filter for TestStatefulFilter {
    fn evaluate(&self, _input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let result = match self.state {
            Mood::Happy => format!(":-) {} :-)", args.arg),
            Mood::Sad => format!(":-( {} :-(", args.arg),
            Mood::Neutral => format!(":-| {} :-|", args.arg),
        };

        Ok(Value::scalar(result))
    }
}
