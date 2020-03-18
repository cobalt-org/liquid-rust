#[macro_use]
extern crate pest_derive;

pub mod error {
    pub use liquid_error::*;
}
pub mod value {
    pub use liquid_value::*;
}

pub mod runtime;
pub mod partials;
pub mod parser;

pub use parser::Language;
pub use parser::TagTokenIter;
pub use parser::{BlockReflection, ParseBlock, TagBlock};
pub use parser::{Filter, FilterParameters, FilterReflection, ParseFilter};
pub use parser::{ParseTag, TagReflection};
pub use liquid_derive::{
    Display_filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
pub use liquid_error::{Error, Result};
pub use runtime::Expression;
pub use runtime::Renderable;
pub use runtime::Runtime;
pub use runtime::Template;
pub use liquid_value::{object, to_object, Object};
pub use liquid_value::{to_value, value, Value, ValueCow};
pub use liquid_value::{ObjectView, ValueView};

#[allow(unused_macros)]
#[macro_export]
macro_rules! call_filter {
    ($filter:expr, $input:expr) => {{
        $crate::call_filter!($filter, $input, )
    }};
    ($filter:expr, $input:expr, $($args:expr),*) => {{
        let positional = Box::new(vec![$($crate::Expression::Literal($crate::value!($args))),*].into_iter());
        let keyword = Box::new(Vec::new().into_iter());
        let args = $crate::parser::FilterArguments { positional, keyword };

        let runtime = $crate::Runtime::default();

        let input = $crate::value!($input);

        $crate::ParseFilter::parse(&$filter, args)
            .and_then(|filter| $crate::Filter::evaluate(&*filter, &input, &runtime))
    }};
}
