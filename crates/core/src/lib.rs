#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::module_inception)]
#![allow(clippy::bool_to_int_with_if)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[macro_use]
extern crate pest_derive;

#[macro_use]
mod macros;

pub mod error;
pub mod model;
pub mod parser;
pub mod partials;
pub mod runtime;

pub use error::{Error, Result};
#[cfg(feature = "derive")]
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
