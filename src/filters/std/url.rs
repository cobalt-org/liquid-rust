use filters::invalid_input;
use liquid_compiler::Filter;
use liquid_derive::*;
use liquid_error::Result;
use liquid_interpreter::Context;
use liquid_value::Value;
use url::percent_encoding;
use url::percent_encoding::EncodeSet;

#[derive(Clone)]
struct UrlEncodeSet(String);

impl UrlEncodeSet {
    fn safe_bytes(&self) -> &[u8] {
        let &UrlEncodeSet(ref safe) = self;
        safe.as_bytes()
    }
}

impl EncodeSet for UrlEncodeSet {
    fn contains(&self, byte: u8) -> bool {
        let is_digit = 48 <= byte && byte <= 57;
        let is_upper = 65 <= byte && byte <= 90;
        let is_lower = 97 <= byte && byte <= 122;
        // -, . or _
        let is_special = byte == 45 || byte == 46 || byte == 95;
        if is_digit || is_upper || is_lower || is_special {
            false
        } else {
            !self.safe_bytes().contains(&byte)
        }
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "url_encode",
    description = "Converts any URL-unsafe characters in a string into percent-encoded characters.",
    parsed(UrlEncodeFilter)
)]
pub struct UrlEncode;

#[derive(Debug, Default, Display_filter)]
#[name = "url_encode"]
struct UrlEncodeFilter;

impl Filter for UrlEncodeFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        lazy_static! {
            static ref URL_ENCODE_SET: UrlEncodeSet = UrlEncodeSet("".to_owned());
        }

        let s = input.to_str();

        let result: String =
            percent_encoding::utf8_percent_encode(s.as_ref(), URL_ENCODE_SET.clone()).collect();
        Ok(Value::scalar(result))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "url_decode",
    description = "Decodes a string that has been encoded as a URL or by url_encode.",
    parsed(UrlDecodeFilter)
)]
pub struct UrlDecode;

#[derive(Debug, Default, Display_filter)]
#[name = "url_decode"]
struct UrlDecodeFilter;

impl Filter for UrlDecodeFilter {
    fn evaluate(&self, input: &Value, _context: &Context) -> Result<Value> {
        let s = input.to_str();

        let result = percent_encoding::percent_decode(s.as_bytes())
            .decode_utf8()
            .map_err(|_| invalid_input("Malformed UTF-8"))?
            .into_owned();
        Ok(Value::scalar(result))
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

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_url_encode() {
        assert_eq!(unit!(UrlEncode, tos!("foo bar")), tos!("foo%20bar"));
        assert_eq!(
            unit!(UrlEncode, tos!("foo+1@example.com")),
            tos!("foo%2B1%40example.com")
        );
    }

    #[test]
    fn unit_url_decode() {
        // TODO Test case from shopify/liquid that we aren't handling:
        // - assert_eq!(unit!(url_decode, tos!("foo+bar")), tos!("foo bar"));
        assert_eq!(unit!(UrlDecode, tos!("foo%20bar")), tos!("foo bar"));
        assert_eq!(
            unit!(UrlDecode, tos!("foo%2B1%40example.com")),
            tos!("foo+1@example.com")
        );
    }
}
