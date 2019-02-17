use liquid_compiler::Filter;
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_value::Value;
use regex::Regex;

/// Returns the number of already escaped characters.
fn nr_escaped(text: &str) -> usize {
    for prefix in &["lt;", "gt;", "#39;", "quot;", "amp;"] {
        if text.starts_with(prefix) {
            return prefix.len();
        }
    }
    0
}

// The code is adapted from
// https://github.com/rust-lang/rust/blob/master/src/librustdoc/html/escape.rs
// Retrieved 2016-11-19.
fn escape(input: &Value, once_p: bool) -> Result<Value> {
    let s = input.to_str();
    let mut result = String::new();
    let mut last = 0;
    let mut skip = 0;
    for (i, c) in s.char_indices() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c as char {
            '<' | '>' | '\'' | '"' | '&' => {
                result.push_str(&s[last..i]);
                last = i + 1;
                let escaped = match c as char {
                    '<' => "&lt;",
                    '>' => "&gt;",
                    '\'' => "&#39;",
                    '"' => "&quot;",
                    '&' => {
                        if once_p {
                            skip = nr_escaped(&s[last..]);
                        }
                        if skip == 0 {
                            "&amp;"
                        } else {
                            "&"
                        }
                    }
                    _ => unreachable!(),
                };
                result.push_str(escaped);
            }
            _ => {}
        }
    }
    if last < s.len() {
        result.push_str(&s[last..]);
    }
    Ok(Value::scalar(result))
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "escape",
    description = "Escapes a string by replacing characters with escape sequences.",
    parsed(EscapeFilter)
)]
pub struct Escape;

#[derive(Debug, Default, Display_filter)]
#[name = "escape"]
struct EscapeFilter;

impl Filter for EscapeFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        escape(input, false)
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "escape_once",
    description = "Escapes a string without changing existing escaped entities.",
    parsed(EscapeOnceFilter)
)]
pub struct EscapeOnce;

#[derive(Debug, Default, Display_filter)]
#[name = "escape_once"]
struct EscapeOnceFilter;

impl Filter for EscapeOnceFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        escape(input, true)
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "strip_html",
    description = "Removes any HTML tags from a string.",
    parsed(StripHtmlFilter)
)]
pub struct StripHtml;

#[derive(Debug, Default, Display_filter)]
#[name = "strip_html"]
struct StripHtmlFilter;

impl Filter for StripHtmlFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        lazy_static! {
            // regexps taken from https://git.io/vXbgS
            static ref MATCHERS: [Regex; 4] = [
                Regex::new(r"(?is)<script.*?</script>").unwrap(),
                Regex::new(r"(?is)<style.*?</style>").unwrap(),
                Regex::new(r"(?is)<!--.*?-->").unwrap(),
                Regex::new(r"(?is)<.*?>").unwrap()
            ];
        }

        let input = input.to_str().into_owned();

        let result = MATCHERS.iter().fold(input, |acc, matcher| {
            matcher.replace_all(&acc, "").into_owned()
        });
        Ok(Value::scalar(result))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "newline_to_br",
    description = "Replaces every newline (`\\n`) with an HTML line break (`<br>`).",
    parsed(NewlineToBrFilter)
)]
pub struct NewlineToBr;

#[derive(Debug, Default, Display_filter)]
#[name = "newline_to_br"]
struct NewlineToBrFilter;

impl Filter for NewlineToBrFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        // TODO handle windows line endings
        let input = input.to_str();
        Ok(Value::scalar(input.replace("\n", "<br />\n")))
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
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            let filter = ::liquid::compiler::ParseFilter::parse(&$a, args).unwrap();
            ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context).unwrap()
        }};
    }

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, )
        }};
        ($a:ident, $b:expr, $($c:expr),*) => {{
            let positional = Box::new(vec![$(::liquid::interpreter::Expression::Literal($c)),*].into_iter());
            let keyword = Box::new(Vec::new().into_iter());
            let args = ::liquid::compiler::FilterArguments { positional, keyword };

            let context = ::liquid::interpreter::Context::default();

            ::liquid::compiler::ParseFilter::parse(&$a, args)
                .and_then(|filter| ::liquid::compiler::Filter::evaluate(&*filter, &$b, &context))
                .unwrap_err()
        }};
    }

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_escape() {
        assert_eq!(
            unit!(Escape, tos!("Have you read 'James & the Giant Peach'?")),
            tos!("Have you read &#39;James &amp; the Giant Peach&#39;?")
        );
        assert_eq!(
            unit!(Escape, tos!("Tetsuro Takara")),
            tos!("Tetsuro Takara")
        );
    }

    #[test]
    fn unit_escape_non_ascii() {
        assert_eq!(
            unit!(Escape, tos!("word¹ <br> word¹")),
            tos!("word¹ &lt;br&gt; word¹")
        );
    }

    #[test]
    fn unit_escape_once() {
        assert_eq!(
            unit!(EscapeOnce, tos!("1 < 2 & 3")),
            tos!("1 &lt; 2 &amp; 3")
        );
        assert_eq!(
            unit!(EscapeOnce, tos!("1 &lt; 2 &amp; 3")),
            tos!("1 &lt; 2 &amp; 3")
        );
        assert_eq!(
            unit!(EscapeOnce, tos!("&lt;&gt;&amp;&#39;&quot;&xyz;")),
            tos!("&lt;&gt;&amp;&#39;&quot;&amp;xyz;")
        );
    }

    #[test]
    fn unit_strip_html() {
        assert_eq!(
            unit!(
                StripHtml,
                tos!("<script type=\"text/javascript\">alert('Hi!');</script>"),
            ),
            tos!("")
        );
        assert_eq!(
            unit!(
                StripHtml,
                tos!("<SCRIPT type=\"text/javascript\">alert('Hi!');</SCRIPT>"),
            ),
            tos!("")
        );
        assert_eq!(unit!(StripHtml, tos!("<p>test</p>")), tos!("test"));
        assert_eq!(unit!(StripHtml, tos!("<p id='xxx'>test</p>")), tos!("test"));
        assert_eq!(
            unit!(
                StripHtml,
                tos!("<style type=\"text/css\">cool style</style>"),
            ),
            tos!("")
        );
        assert_eq!(
            unit!(StripHtml, tos!("<p\nclass='loooong'>test</p>")),
            tos!("test")
        );
        assert_eq!(
            unit!(StripHtml, tos!("<!--\n\tcomment\n-->test")),
            tos!("test")
        );
        assert_eq!(unit!(StripHtml, tos!("")), tos!(""));
    }

    #[test]
    fn unit_newline_to_br() {
        let input = &tos!("a\nb");
        let desired_result = tos!("a<br />\nb");
        assert_eq!(unit!(NewlineToBr, input), desired_result);
    }

    #[test]
    fn unit_newline_to_br_hello_world() {
        // First example from https://shopify.github.io/liquid/filters/newline_to_br/
        let input = &tos!("\nHello\nWorld\n");
        let desired_result = tos!("<br />\nHello<br />\nWorld<br />\n");
        assert_eq!(unit!(NewlineToBr, input), desired_result);
    }

    #[test]
    fn unit_newline_to_br_one_argument() {
        let input = &tos!("a\nb");
        failed!(NewlineToBr, input, Value::scalar(0f64));
    }
}
