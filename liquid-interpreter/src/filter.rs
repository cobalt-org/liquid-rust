use liquid_error;

use value::Value;

/// Expected return type of a `Filter`.
pub type FilterResult = Result<Value, liquid_error::Error>;

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [`LiquidOptions`](struct.LiquidOptions.html).
pub trait FilterValue: Send + Sync + FilterValueClone {
    /// Filter `input` based on `arguments`.
    fn filter(&self, input: &Value, arguments: &[Value]) -> FilterResult;
}

/// Support cloning of `Box<FilterValue>`.
pub trait FilterValueClone {
    /// Cloning of `dyn FilterValue`.
    fn clone_box(&self) -> Box<FilterValue>;
}

impl<T> FilterValueClone for T
where
    T: 'static + FilterValue + Clone,
{
    fn clone_box(&self) -> Box<FilterValue> {
        Box::new(self.clone())
    }
}

impl Clone for Box<FilterValue> {
    fn clone(&self) -> Box<FilterValue> {
        self.clone_box()
    }
}

/// Function signature that can act as a `FilterValue`.
pub type FnFilterValue = fn(&Value, &[Value]) -> FilterResult;

#[derive(Clone)]
struct FnValueFilter {
    filter: FnFilterValue,
}

impl FnValueFilter {
    fn new(filter: FnFilterValue) -> Self {
        Self { filter }
    }
}

impl FilterValue for FnValueFilter {
    fn filter(&self, input: &Value, arguments: &[Value]) -> FilterResult {
        (self.filter)(input, arguments)
    }
}

#[derive(Clone)]
enum EnumValueFilter {
    Fun(FnValueFilter),
    Heap(Box<FilterValue>),
}

/// Custom `Box<FilterValue>` with a `FnFilterValue` optimization.
#[derive(Clone)]
pub struct BoxedValueFilter {
    filter: EnumValueFilter,
}

impl FilterValue for BoxedValueFilter {
    fn filter(&self, input: &Value, arguments: &[Value]) -> FilterResult {
        match self.filter {
            EnumValueFilter::Fun(ref f) => f.filter(input, arguments),
            EnumValueFilter::Heap(ref f) => f.filter(input, arguments),
        }
    }
}

impl From<fn(&Value, &[Value]) -> FilterResult> for BoxedValueFilter {
    fn from(filter: FnFilterValue) -> BoxedValueFilter {
        let filter = EnumValueFilter::Fun(FnValueFilter::new(filter));
        Self { filter }
    }
}

impl From<Box<FilterValue>> for BoxedValueFilter {
    fn from(filter: Box<FilterValue>) -> BoxedValueFilter {
        let filter = EnumValueFilter::Heap(filter);
        Self { filter }
    }
}
