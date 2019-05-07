use liquid_error::Result;
use liquid_interpreter::Renderable;

use super::Language;
use super::TagTokenIter;

pub trait TagReflection {
    fn tag(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn example(&self) -> Option<&'static str> {
        None
    }

    fn spec(&self) -> Option<&'static str> {
        None
    }
}

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [`Language`](struct.Language.html).
pub trait ParseTag: Send + Sync + ParseTagClone + TagReflection {
    fn parse(&self, arguments: TagTokenIter, options: &Language) -> Result<Box<Renderable>>;
}

pub trait ParseTagClone {
    fn clone_box(&self) -> Box<ParseTag>;
}

impl<T> ParseTagClone for T
where
    T: 'static + ParseTag + Clone,
{
    fn clone_box(&self) -> Box<ParseTag> {
        Box::new(self.clone())
    }
}

impl Clone for Box<ParseTag> {
    fn clone(&self) -> Box<ParseTag> {
        self.clone_box()
    }
}

impl<T> From<T> for Box<ParseTag>
where
    T: 'static + ParseTag,
{
    fn from(filter: T) -> Self {
        Box::new(filter)
    }
}
