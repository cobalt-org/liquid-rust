use std::cmp;
use std::fmt::Write;

use liquid_core::model::try_find;
use liquid_core::model::KStringCow;
use liquid_core::model::ValueViewCmp;
use liquid_core::parser::parse_variable;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueCow;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

use crate::invalid_input;

#[derive(Debug, Default, FilterParameters)]
struct SortArgs {
    #[parameter(description = "The property accessed by the filter.", arg_type = "str")]
    property: Option<Expression>,
    #[parameter(
        description = "nils appear before or after non-nil values, either ('first' | 'last')",
        arg_type = "str"
    )]
    nils: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "sort",
    description = "Sorts items in an array. The order of the sorted array is case-sensitive.",
    parameters(SortArgs),
    parsed(SortFilter)
)]
pub struct Sort;

#[derive(Debug, Default, FromFilterParameters, Display_filter)]
#[name = "sort"]
struct SortFilter {
    #[parameters]
    args: SortArgs,
}

#[derive(Copy, Clone)]
enum NilsOrder {
    First,
    Last,
}

fn safe_property_getter<'v>(
    value: &'v Value,
    property: &KStringCow<'_>,
    runtime: &dyn Runtime,
) -> ValueCow<'v> {
    let variable = parse_variable(property).expect("Failed to parse variable");
    if let Some(path) = variable.try_evaluate(runtime) {
        try_find(value, path.as_slice()).unwrap_or(ValueCow::Borrowed(&Value::Nil))
    } else {
        ValueCow::Borrowed(&Value::Nil)
    }
}

fn nil_safe_compare(
    a: &dyn ValueView,
    b: &dyn ValueView,
    nils: NilsOrder,
) -> Option<cmp::Ordering> {
    if a.is_nil() && b.is_nil() {
        Some(cmp::Ordering::Equal)
    } else if a.is_nil() {
        match nils {
            NilsOrder::First => Some(cmp::Ordering::Less),
            NilsOrder::Last => Some(cmp::Ordering::Greater),
        }
    } else if b.is_nil() {
        match nils {
            NilsOrder::First => Some(cmp::Ordering::Greater),
            NilsOrder::Last => Some(cmp::Ordering::Less),
        }
    } else {
        ValueViewCmp::new(a).partial_cmp(&ValueViewCmp::new(b))
    }
}

fn as_sequence<'k>(input: &'k dyn ValueView) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
    if let Some(array) = input.as_array() {
        array.values()
    } else if input.is_nil() {
        Box::new(vec![].into_iter())
    } else {
        Box::new(std::iter::once(input))
    }
}

impl Filter for SortFilter {
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let input: Vec<_> = as_sequence(input).collect();
        if input.is_empty() {
            return Err(invalid_input("Non-empty array expected"));
        }
        if args.property.is_some() && !input.iter().all(|v| v.is_object()) {
            return Err(invalid_input("Array of objects expected"));
        }
        let nils = if let Some(nils) = &args.nils {
            match nils.to_kstr().as_str() {
                "first" => NilsOrder::First,
                "last" => NilsOrder::Last,
                _ => {
                    return Err(invalid_input(
                        "Invalid nils order. Must be \"first\" or \"last\".",
                    ))
                }
            }
        } else {
            NilsOrder::First
        };

        let mut sorted: Vec<Value> = input.iter().map(|v| v.to_value()).collect();
        if let Some(property) = &args.property {
            // Using unwrap is ok since all of the elements are objects
            sorted.sort_by(|a, b| {
                nil_safe_compare(
                    safe_property_getter(a, property, runtime).as_view(),
                    safe_property_getter(b, property, runtime).as_view(),
                    nils,
                )
                .unwrap_or(cmp::Ordering::Equal)
            });
        } else {
            sorted.sort_by(|a, b| nil_safe_compare(a, b, nils).unwrap_or(cmp::Ordering::Equal));
        }
        Ok(Value::array(sorted))
    }
}

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
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let element = args.element.to_value();
        let mut array = input
            .to_value()
            .into_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        array.push(element);

        Ok(Value::Array(array))
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
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        let mut array = input
            .to_value()
            .into_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        array.pop();

        Ok(Value::Array(array))
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
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let element = args.element.to_value();
        let mut array = input
            .to_value()
            .into_array()
            .ok_or_else(|| invalid_input("Array expected"))?;
        array.insert(0, element);

        Ok(Value::Array(array))
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
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        let mut array = input
            .to_value()
            .into_array()
            .ok_or_else(|| invalid_input("Array expected"))?;

        if !array.is_empty() {
            array.remove(0);
        }

        Ok(Value::Array(array))
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
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let connector = args.connector.unwrap_or_else(|| "and".into());

        let mut array = input
            .as_array()
            .ok_or_else(|| invalid_input("Array expected"))?
            .values();

        let mut sentence = array
            .next()
            .map(|v| v.to_kstr().into_string())
            .unwrap_or_else(|| "".to_owned());

        let mut iter = array.peekable();
        while let Some(value) = iter.next() {
            if iter.peek().is_some() {
                write!(sentence, ", {}", value.render())
                    .expect("It should be safe to write to a string.");
            } else {
                write!(sentence, ", {} {}", connector, value.render())
                    .expect("It should be safe to write to a string.");
            }
        }

        Ok(Value::scalar(sentence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_sort() {
        let input = &liquid_core::value!(["Z", "b", "c", "a"]);
        let desired_result = liquid_core::value!(["Z", "a", "b", "c"]);
        assert_eq!(
            liquid_core::call_filter!(Sort, input).unwrap(),
            desired_result
        );
    }

    #[test]
    fn unit_push() {
        let input = liquid_core::value!(["Seattle", "Tacoma"]);
        let unit_result = liquid_core::call_filter!(Push, input, "Spokane").unwrap();
        let desired_result = liquid_core::value!(["Seattle", "Tacoma", "Spokane",]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_pop() {
        let input = liquid_core::value!(["Seattle", "Tacoma"]);
        let unit_result = liquid_core::call_filter!(Pop, input).unwrap();
        let desired_result = liquid_core::value!(["Seattle"]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_pop_empty() {
        let input = liquid_core::value!([]);
        let unit_result = liquid_core::call_filter!(Pop, input).unwrap();
        let desired_result = liquid_core::value!([]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_unshift() {
        let input = liquid_core::value!(["Seattle", "Tacoma"]);
        let unit_result = liquid_core::call_filter!(Unshift, input, "Olympia").unwrap();
        let desired_result = liquid_core::value!(["Olympia", "Seattle", "Tacoma"]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_shift() {
        let input = liquid_core::value!(["Seattle", "Tacoma"]);
        let unit_result = liquid_core::call_filter!(Shift, input).unwrap();
        let desired_result = liquid_core::value!(["Tacoma"]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_shift_empty() {
        let input = liquid_core::value!([]);
        let unit_result = liquid_core::call_filter!(Shift, input).unwrap();
        let desired_result = liquid_core::value!([]);
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string() {
        let input = liquid_core::value!(["foo", "bar", "baz"]);
        let unit_result = liquid_core::call_filter!(ArrayToSentenceString, input).unwrap();
        let desired_result = "foo, bar, and baz";
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_two_elements() {
        let input = liquid_core::value!(["foo", "bar"]);
        let unit_result = liquid_core::call_filter!(ArrayToSentenceString, input).unwrap();
        let desired_result = "foo, and bar";
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_one_element() {
        let input = liquid_core::value!(["foo"]);
        let unit_result = liquid_core::call_filter!(ArrayToSentenceString, input).unwrap();
        let desired_result = "foo";
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_no_elements() {
        let input = liquid_core::value!([]);
        let unit_result = liquid_core::call_filter!(ArrayToSentenceString, input).unwrap();
        let desired_result = "";
        assert_eq!(unit_result, desired_result);
    }

    #[test]
    fn unit_array_to_sentence_string_custom_connector() {
        let input = liquid_core::value!(["foo", "bar", "baz"]);
        let unit_result = liquid_core::call_filter!(ArrayToSentenceString, input, "or").unwrap();
        let desired_result = "foo, bar, or baz";
        assert_eq!(unit_result, desired_result);
    }
}
