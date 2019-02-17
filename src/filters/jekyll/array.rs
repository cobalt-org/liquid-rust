use filters::invalid_input;
use liquid_compiler::{Filter, FilterParameters};
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_interpreter::Expression;
use liquid_value::Value;
use std::fmt::Write;

#[derive(Debug, FilterParameters)]
struct PushArgs {
    #[parameter(description = "The element to append to the array.")]
    element: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "push",
    description = "Appends the given element to the end of an array.",
    parameters(PushArgs),
    parsed(PushFilter)
)]
pub struct Push;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "push"]
struct PushFilter {
    #[parameters]
    args: PushArgs,
}

impl Filter for PushFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let element = args.element.clone();
        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .clone();
        array.push(element);

        Ok(Value::array(array))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "pop",
    description = "Removes the last element of an array.",
    parsed(PopFilter)
)]
pub struct Pop;

#[derive(Debug, Default, Display_filter)]
#[name = "pop"]
struct PopFilter;

impl Filter for PopFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .clone();
        array.pop();

        Ok(Value::array(array))
    }
}

#[derive(Debug, FilterParameters)]
struct UnshiftArgs {
    #[parameter(description = "The element to append to the array.")]
    element: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "unshift",
    description = "Appends the given element to the start of an array.",
    parameters(UnshiftArgs),
    parsed(UnshiftFilter)
)]
pub struct Unshift;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "unshift"]
struct UnshiftFilter {
    #[parameters]
    args: UnshiftArgs,
}

impl Filter for UnshiftFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let element = args.element.clone();
        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .clone();
        array.insert(0, element);

        Ok(Value::array(array))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "shift",
    description = "Removes the first element of an array.",
    parsed(ShiftFilter)
)]
pub struct Shift;

#[derive(Debug, Default, Display_filter)]
#[name = "shift"]
struct ShiftFilter;

impl Filter for ShiftFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .clone();

        if !array.is_empty() {
            array.remove(0);
        }

        Ok(Value::array(array))
    }
}

#[derive(Debug, FilterParameters)]
struct ArrayToSentenceStringArgs {
    #[parameter(
        description = "The connector between the last two elements. Defaults to \"and\".",
        arg_type = "str"
    )]
    connector: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "array_to_sentence_string",
    description = "Converts an array into a sentence. This sentence will be a list of the elements of the array separated by comma, with a connector between the last two elements.",
    parameters(ArrayToSentenceStringArgs),
    parsed(ArrayToSentenceStringFilter)
)]
pub struct ArrayToSentenceString;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "array_to_sentence_string"]
struct ArrayToSentenceStringFilter {
    #[parameters]
    args: ArrayToSentenceStringArgs,
}

impl Filter for ArrayToSentenceStringFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let connector = args.connector.unwrap_or("and".into());

        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .iter();

        let mut sentence = array
            .next()
            .map(|v| v.to_str().into_owned())
            .unwrap_or_else(|| "".to_string());

        let last = array.next_back();

        for value in array {
            write!(sentence, ", {}", value.render())
                .expect("It should be safe to write to a string.");
        }

        if let Some(last) = last {
            write!(sentence, ", {} {}", connector, last.render())
                .expect("It should be safe to write to a string.");
        }

        Ok(Value::scalar(sentence))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            let filter = ::liquid::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    #[test]
    fn unit_push() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let unit_result = unit!(Push, input, Value::scalar("Spokane"));
        let desired_result = Value::Array(vec![
            Value::scalar("Seattle"),
            Value::scalar("Tacoma"),
            Value::scalar("Spokane"),
        ]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_pop() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let unit_result = unit!(Pop, input);
        let desired_result = Value::Array(vec![Value::scalar("Seattle")]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_pop_empty() {
        let input = Value::Array(vec![]);
        let unit_result = unit!(Pop, input);
        let desired_result = Value::Array(vec![]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_unshift() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let unit_result = unit!(Unshift, input, Value::scalar("Olympia"));
        let desired_result = Value::Array(vec![
            Value::scalar("Olympia"),
            Value::scalar("Seattle"),
            Value::scalar("Tacoma"),
        ]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_shift() {
        let input = Value::Array(vec![Value::scalar("Seattle"), Value::scalar("Tacoma")]);
        let unit_result = unit!(Shift, input);
        let desired_result = Value::Array(vec![Value::scalar("Tacoma")]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_shift_empty() {
        let input = Value::Array(vec![]);
        let unit_result = unit!(Shift, input);
        let desired_result = Value::Array(vec![]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string() {
        let input = Value::Array(vec![
            Value::scalar("foo"),
            Value::scalar("bar"),
            Value::scalar("baz"),
        ]);
        let unit_result = unit!(ArrayToSentenceString, input);
        let desired_result = Value::scalar("foo, bar, and baz");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_two_elements() {
        let input = Value::Array(vec![Value::scalar("foo"), Value::scalar("bar")]);
        let unit_result = unit!(ArrayToSentenceString, input);
        let desired_result = Value::scalar("foo, and bar");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_one_element() {
        let input = Value::Array(vec![Value::scalar("foo")]);
        let unit_result = unit!(ArrayToSentenceString, input);
        let desired_result = Value::scalar("foo");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_no_elements() {
        let input = Value::Array(vec![]);
        let unit_result = unit!(ArrayToSentenceString, input);
        let desired_result = Value::scalar("");
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_custom_connector() {
        let input = Value::Array(vec![
            Value::scalar("foo"),
            Value::scalar("bar"),
            Value::scalar("baz"),
        ]);
        let unit_result = unit!(ArrayToSentenceString, input, Value::scalar("or"));
        let desired_result = Value::scalar("foo, bar, or baz");
        assert_eq!(unit_result, desired_result);
    }
}
