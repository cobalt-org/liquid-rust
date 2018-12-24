use regex::Regex;

use liquid_value::Value;

use super::check_args_len;
use compiler::FilterResult;

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
fn _escape(input: &Value, args: &[Value], once_p: bool) -> FilterResult {
    check_args_len(args, 0, 0)?;

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

pub fn escape(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, false)
}

pub fn escape_once(input: &Value, args: &[Value]) -> FilterResult {
    _escape(input, args, true)
}

pub fn strip_html(input: &Value, args: &[Value]) -> FilterResult {
    lazy_static! {
        // regexps taken from https://git.io/vXbgS
        static ref MATCHERS: [Regex; 4] = [Regex::new(r"(?is)<script.*?</script>").unwrap(),
                                           Regex::new(r"(?is)<style.*?</style>").unwrap(),
                                           Regex::new(r"(?is)<!--.*?-->").unwrap(),
                                           Regex::new(r"(?is)<.*?>").unwrap()];
    }
    check_args_len(args, 0, 0)?;

    let input = input.to_str().into_owned();

    let result = MATCHERS.iter().fold(input, |acc, matcher| {
        matcher.replace_all(&acc, "").into_owned()
    });
    Ok(Value::scalar(result))
}

/// Replaces every newline (`\n`) with an HTML line break (`<br>`).
pub fn newline_to_br(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    // TODO handle windows line endings
    let input = input.to_str();
    Ok(Value::scalar(input.replace("\n", "<br />\n")))
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ($a:ident, $b:expr) => {{
            unit!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    macro_rules! failed {
        ($a:ident, $b:expr) => {{
            failed!($a, $b, &[])
        }};
        ($a:ident, $b:expr, $c:expr) => {{
            $a(&$b, $c).unwrap_err()
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
            unit!(escape, tos!("Have you read 'James & the Giant Peach'?")),
            tos!("Have you read &#39;James &amp; the Giant Peach&#39;?")
        );
        assert_eq!(
            unit!(escape, tos!("Tetsuro Takara")),
            tos!("Tetsuro Takara")
        );
    }

    #[test]
    fn unit_escape_non_ascii() {
        assert_eq!(
            unit!(escape, tos!("word¹ <br> word¹")),
            tos!("word¹ &lt;br&gt; word¹")
        );
    }

    #[test]
    fn unit_escape_once() {
        assert_eq!(
            unit!(escape_once, tos!("1 < 2 & 3")),
            tos!("1 &lt; 2 &amp; 3")
        );
        assert_eq!(
            unit!(escape_once, tos!("1 &lt; 2 &amp; 3")),
            tos!("1 &lt; 2 &amp; 3")
        );
        assert_eq!(
            unit!(escape_once, tos!("&lt;&gt;&amp;&#39;&quot;&xyz;")),
            tos!("&lt;&gt;&amp;&#39;&quot;&amp;xyz;")
        );
    }

    #[test]
    fn unit_strip_html() {
        assert_eq!(
            unit!(
                strip_html,
                tos!("<script type=\"text/javascript\">alert('Hi!');</script>"),
                &[]
            ),
            tos!("")
        );
        assert_eq!(
            unit!(
                strip_html,
                tos!("<SCRIPT type=\"text/javascript\">alert('Hi!');</SCRIPT>"),
                &[]
            ),
            tos!("")
        );
        assert_eq!(unit!(strip_html, tos!("<p>test</p>"), &[]), tos!("test"));
        assert_eq!(
            unit!(strip_html, tos!("<p id='xxx'>test</p>"), &[]),
            tos!("test")
        );
        assert_eq!(
            unit!(
                strip_html,
                tos!("<style type=\"text/css\">cool style</style>"),
                &[]
            ),
            tos!("")
        );
        assert_eq!(
            unit!(strip_html, tos!("<p\nclass='loooong'>test</p>"), &[]),
            tos!("test")
        );
        assert_eq!(
            unit!(strip_html, tos!("<!--\n\tcomment\n-->test"), &[]),
            tos!("test")
        );
        assert_eq!(unit!(strip_html, tos!(""), &[]), tos!(""));
    }

    #[test]
    fn unit_newline_to_br() {
        let input = &tos!("a\nb");
        let args = &[];
        let desired_result = tos!("a<br />\nb");
        assert_eq!(unit!(newline_to_br, input, args), desired_result);
    }

    #[test]
    fn unit_newline_to_br_hello_world() {
        // First example from https://shopify.github.io/liquid/filters/newline_to_br/
        let input = &tos!("\nHello\nWorld\n");
        let args = &[];
        let desired_result = tos!("<br />\nHello<br />\nWorld<br />\n");
        assert_eq!(unit!(newline_to_br, input, args), desired_result);
    }

    #[test]
    fn unit_newline_to_br_one_argument() {
        let input = &tos!("a\nb");
        let args = &[Value::scalar(0f64)];
        failed!(newline_to_br, input, args);
    }
}
