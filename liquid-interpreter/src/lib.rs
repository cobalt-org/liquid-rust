#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate liquid_error;
extern crate liquid_value;

#[cfg(test)]
extern crate serde_yaml;

// Minimize retrofits
mod error {
    pub(crate) use liquid_error::*;
}
mod value {
    pub(crate) use liquid_value::*;
}

mod argument;
mod context;
mod filter;
mod globals;
mod output;
mod renderable;
mod template;
mod text;
mod variable;

pub use self::argument::Argument;
pub use self::context::{
    unexpected_value_error, Context, ContextBuilder, Interrupt, InterruptState,
};
pub use self::filter::{BoxedValueFilter, FilterError, FilterResult, FilterValue, FnFilterValue};
pub use self::globals::Globals;
pub use self::output::{FilterPrototype, Output};
pub use self::renderable::Renderable;
pub use self::template::Template;
pub use self::text::Text;
pub use self::variable::Variable;
