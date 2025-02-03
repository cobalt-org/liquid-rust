use liquid_core::parser;

pub use parser::BlockReflection;
pub use parser::FilterReflection;
pub use parser::TagReflection;

pub trait ParserReflection {
    fn blocks(&self) -> Box<dyn Iterator<Item = &dyn BlockReflection> + '_>;

    fn tags(&self) -> Box<dyn Iterator<Item = &dyn TagReflection> + '_>;

    fn filters(&self) -> Box<dyn Iterator<Item = &dyn FilterReflection> + '_>;

    fn partials(&self) -> Box<dyn Iterator<Item = &str> + '_>;
}
