use LiquidOptions;
use error::Result;

use super::Token;
use super::Renderable;

/// A trait for creating custom tags. This is a simple type alias for a function.
///
/// This function will be called whenever the parser encounters a tag and returns
/// a new [Renderable](trait.Renderable.html) based on its parameters. The received parameters
/// specify the name of the tag, the argument [Tokens](lexer/enum.Token.html) passed to
/// the tag and the global [`LiquidOptions`](struct.LiquidOptions.html).
///
/// ## Minimal Example
/// ```
/// # use liquid::{Renderable, LiquidOptions, Context, Error, FnTagParser};
///
/// struct HelloWorld;
///
/// impl Renderable for HelloWorld {
///     fn render(&self, _context: &mut Context) -> Result<Option<String>, Error>{
///         Ok(Some("Hello World!".to_owned()))
///     }
/// }
///
/// let mut options : LiquidOptions = Default::default();
/// options.tags.insert(
///     "hello_world".to_owned(),
///     Box::new(FnTagParser::new(|_tag_name, _arguments, _options| {
///         Ok(Box::new(HelloWorld))
///     })),
/// );
///
/// let template = liquid::parse("{{hello_world}}", options).unwrap();
/// let mut data = Context::new();
/// let output = template.render(&mut data);
/// assert_eq!(output.unwrap(), Some("Hello World!".to_owned()));
/// ```
pub trait ParseTag: Send + Sync + ParseTagClone {
    fn parse(&self,
             tag_name: &str,
             arguments: &[Token],
             options: &LiquidOptions)
             -> Result<Box<Renderable>>;
}

pub trait ParseTagClone {
    fn clone_box(&self) -> Box<ParseTag>;
}

impl<T> ParseTagClone for T
    where T: 'static + ParseTag + Clone
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

pub type FnParseTag = fn(&str, &[Token], &LiquidOptions) -> Result<Box<Renderable>>;

#[derive(Clone)]
pub struct FnTagParser {
    pub parser: FnParseTag,
}

impl FnTagParser {
    pub fn new(parser: FnParseTag) -> Self {
        Self { parser }
    }
}

impl ParseTag for FnTagParser {
    fn parse(&self,
             tag_name: &str,
             arguments: &[Token],
             options: &LiquidOptions)
             -> Result<Box<Renderable>> {
        (self.parser)(tag_name, arguments, options)
    }
}
