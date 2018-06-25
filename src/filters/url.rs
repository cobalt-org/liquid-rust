use url::percent_encoding;
use url::percent_encoding::EncodeSet;

use interpreter::{FilterError, FilterResult};
use value::Value;

use super::check_args_len;

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

pub fn url_encode(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    lazy_static! {
        static ref URL_ENCODE_SET: UrlEncodeSet = UrlEncodeSet("".to_owned());
    }

    let s = input.to_str();

    let result: String =
        percent_encoding::utf8_percent_encode(s.as_ref(), URL_ENCODE_SET.clone()).collect();
    Ok(Value::scalar(result))
}

pub fn url_decode(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 0)?;

    let s = input.to_str();

    let result = percent_encoding::percent_decode(s.as_bytes())
        .decode_utf8()
        .map_err(|_| FilterError::InvalidType("Malformed UTF-8".to_owned()))?
        .into_owned();
    Ok(Value::scalar(result))
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

    macro_rules! tos {
        ($a:expr) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_url_encode() {
        assert_eq!(unit!(url_encode, tos!("foo bar")), tos!("foo%20bar"));
        assert_eq!(
            unit!(url_encode, tos!("foo+1@example.com")),
            tos!("foo%2B1%40example.com")
        );
    }

    #[test]
    fn unit_url_decode() {
        // TODO Test case from shopify/liquid that we aren't handling:
        // - assert_eq!(unit!(url_decode, tos!("foo+bar")), tos!("foo bar"));
        assert_eq!(unit!(url_decode, tos!("foo%20bar")), tos!("foo bar"));
        assert_eq!(
            unit!(url_decode, tos!("foo%2B1%40example.com")),
            tos!("foo+1@example.com")
        );
    }
}
