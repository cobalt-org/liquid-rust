pub mod compiler {
    pub use liquid_compiler::*;
}
pub mod error {
    pub use liquid_error::*;
}
pub mod interpreter {
    pub use liquid_interpreter::*;
}
pub mod value {
    pub use liquid_value::*;
}

pub use liquid_compiler::Language;
pub use liquid_compiler::TagTokenIter;
pub use liquid_compiler::{BlockReflection, ParseBlock, TagBlock};
pub use liquid_compiler::{Filter, FilterParameters, FilterReflection, ParseFilter};
pub use liquid_compiler::{ParseTag, TagReflection};
pub use liquid_derive::{
    Display_filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
pub use liquid_error::{Error, Result};
pub use liquid_interpreter::Expression;
pub use liquid_interpreter::Renderable;
pub use liquid_interpreter::Runtime;
pub use liquid_interpreter::Template;
pub use liquid_value::{object, to_object, Object};
pub use liquid_value::{to_value, value, Value};
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
        let args = $crate::compiler::FilterArguments { positional, keyword };

        let runtime = $crate::Runtime::default();

        let input = $crate::value!($input);

        $crate::ParseFilter::parse(&$filter, args)
            .and_then(|filter| $crate::Filter::evaluate(&*filter, &input, &runtime))
    }};
}
