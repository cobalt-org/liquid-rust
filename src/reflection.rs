use liquid_core::compiler;

pub use compiler::BlockReflection;
pub use compiler::FilterReflection;
pub use compiler::TagReflection;

pub trait ParserReflection {
    fn blocks<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::BlockReflection> + 'r>;

    fn tags<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::TagReflection> + 'r>;

    fn filters<'r>(&'r self) -> Box<dyn Iterator<Item = &dyn compiler::FilterReflection> + 'r>;

    fn partials<'r>(&'r self) -> Box<dyn Iterator<Item = &str> + 'r>;
}
