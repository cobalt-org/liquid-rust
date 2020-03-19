#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod model;
pub mod parser;
pub mod partials;
pub mod runtime;

pub use error::{Error, Result};
#[doc(hidden)]
pub use liquid_derive::{
    Display_filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
pub use model::{to_object, Object};
pub use model::{to_value, Value, ValueCow};
pub use model::{ObjectView, ValueView};
pub use parser::Language;
pub use parser::TagTokenIter;
pub use parser::{BlockReflection, ParseBlock, TagBlock};
pub use parser::{Filter, FilterParameters, FilterReflection, ParseFilter};
pub use parser::{ParseTag, TagReflection};
pub use runtime::Expression;
pub use runtime::Renderable;
pub use runtime::Runtime;
pub use runtime::Template;

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
