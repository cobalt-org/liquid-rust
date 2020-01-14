use deunicode;
use liquid_core::Context;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};
use regex::Regex;

#[derive(PartialEq)]
enum SlugifyMode {
    No,
    Def,
    Raw,
    Pretty,
    Ascii,
    Latin,
}

impl SlugifyMode {
    fn new(mode_str: &str) -> SlugifyMode {
        match mode_str {
            "none" => SlugifyMode::No,
            "raw" => SlugifyMode::Raw,
            "pretty" => SlugifyMode::Pretty,
            "ascii" => SlugifyMode::Ascii,
            "latin" => SlugifyMode::Latin,
            _ => SlugifyMode::Def,
        }
    }
}

static SLUG_INVALID_CHARS_DEFAULT: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"([^0-9\p{Alphabetic}]+)").unwrap());
static SLUG_INVALID_CHARS_RAW: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"([\s]+)").unwrap());
static SLUG_INVALID_CHARS_PRETTY: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
    Regex::new(r"([^\p{Alphabetic}0-9\._\~!\$&'\(\)\+,;=@]+)").unwrap()
});
static SLUG_INVALID_CHARS_ASCII: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"([^a-zA-Z0-9]+)").unwrap());

#[derive(Debug, FilterParameters)]
struct SlugifyArgs {
    #[parameter(
        description = "The slugify mode. May be \"none\", \"raw\", \"pretty\", \"ascii\", \"latin\" or \"default\".",
        arg_type = "str"
    )]
    mode: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "slugify",
    description = "Convert a string into a lowercase URL \"slug\".",
    parameters(SlugifyArgs),
    parsed(SlugifyFilter)
)]
pub struct Slugify;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "slugify"]
struct SlugifyFilter {
    #[parameters]
    args: SlugifyArgs,
}

impl Filter for SlugifyFilter {
    fn evaluate(&self, input: &Value, context: &Context) -> Result<Value> {
        let args = self.args.evaluate(context)?;

        let s = input.to_kstr();
        let mode = args
            .mode
            .map(|mode| SlugifyMode::new(mode.as_str()))
            .unwrap_or(SlugifyMode::Def);

        let s = if mode == SlugifyMode::Latin {
            deunicode::deunicode_with_tofu(&s.trim(), "-")
        } else {
            s.trim().to_string()
        };

        let result = match mode {
            SlugifyMode::No => s,
            SlugifyMode::Def => SLUG_INVALID_CHARS_DEFAULT.replace_all(&s, "-").to_string(),
            SlugifyMode::Raw => SLUG_INVALID_CHARS_RAW.replace_all(&s, "-").to_string(),
            SlugifyMode::Pretty => SLUG_INVALID_CHARS_PRETTY.replace_all(&s, "-").to_string(),
            SlugifyMode::Ascii | SlugifyMode::Latin => {
                SLUG_INVALID_CHARS_ASCII.replace_all(&s, "-").to_string()
            }
        };

        Ok(Value::scalar(result.trim_matches('-').to_lowercase()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid_core::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid_core::compiler::FilterArguments { positional, keyword };

            let context = ::liquid_core::interpreter::Context::default();

            let filter = ::liquid_core::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid_core::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn test_slugify_default() {
        assert_eq!(
            unit!(Slugify, tos!("The _cönfig.yml file")),
            tos!("the-cönfig-yml-file")
        );
    }

    #[test]
    fn test_slugify_ascii() {
        assert_eq!(
            unit!(Slugify, tos!("The _cönfig.yml file"), tos!("ascii")),
            tos!("the-c-nfig-yml-file")
        );
    }

    #[test]
    fn test_slugify_latin() {
        assert_eq!(
            unit!(Slugify, tos!("The _cönfig.yml file"), tos!("latin")),
            tos!("the-config-yml-file")
        );
    }

    #[test]
    fn test_slugify_raw() {
        assert_eq!(
            unit!(Slugify, tos!("The _config.yml file"), tos!("raw")),
            tos!("the-_config.yml-file")
        );
    }

    #[test]
    fn test_slugify_none() {
        assert_eq!(
            unit!(Slugify, tos!("The _config.yml file"), tos!("none")),
            tos!("the _config.yml file")
        );
    }
}
