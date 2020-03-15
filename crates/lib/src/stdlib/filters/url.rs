use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{Display_filter, Filter, FilterReflection, ParseFilter};
use liquid_core::{Value, ValueView};
use url::percent_encoding;
use url::percent_encoding::EncodeSet;

use crate::invalid_input;

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

static URL_ENCODE_SET: once_cell::sync::Lazy<UrlEncodeSet> =
    once_cell::sync::Lazy::new(|| UrlEncodeSet("".to_owned()));

impl Filter for UrlEncodeFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &Runtime<'_>) -> Result<Value> {
        if input.is_nil() {
            return Ok(Value::Nil);
        }

        let s = input.to_kstr();

        let result: String =
            percent_encoding::utf8_percent_encode(s.as_str(), URL_ENCODE_SET.clone()).collect();
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
    fn evaluate(&self, input: &dyn ValueView, _runtime: &Runtime<'_>) -> Result<Value> {
        if input.is_nil() {
            return Ok(Value::Nil);
        }

        let s = input.to_kstr().replace("+", " ");

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

    #[test]
    fn unit_url_encode() {
        assert_eq!(
            liquid_core::call_filter!(UrlEncode, "foo bar").unwrap(),
            liquid_core::value!("foo%20bar")
        );
        assert_eq!(
            liquid_core::call_filter!(UrlEncode, "foo+1@example.com").unwrap(),
            liquid_core::value!("foo%2B1%40example.com")
        );
    }

    #[test]
    fn unit_url_decode() {
        // TODO Test case from shopify/liquid that we aren't handling:
        // - assert_eq!(
        //      liquid_core::call_filter!(url_decode, "foo+bar").unwrap(),
        //      liquid_core::value!("foo bar")
        //  );
        assert_eq!(
            liquid_core::call_filter!(UrlDecode, "foo%20bar").unwrap(),
            liquid_core::value!("foo bar")
        );
        assert_eq!(
            liquid_core::call_filter!(UrlDecode, "foo%2B1%40example.com").unwrap(),
            liquid_core::value!("foo+1@example.com")
        );
    }
}
