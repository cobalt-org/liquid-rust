use deunicode;
use liquid_value::Value;
use regex::Regex;

use super::check_args_len;

use compiler::FilterResult;

#[derive(PartialEq)]
enum SlugifyMode {
    No,
    Def,
    Raw,
    Pretty,
    Ascii,
    Latin,
}

lazy_static! {
    static ref SLUG_INVALID_CHARS_DEFAULT: Regex = Regex::new(r"([^0-9\p{Alphabetic}]+)").unwrap();
    static ref SLUG_INVALID_CHARS_RAW: Regex = Regex::new(r"([\s]+)").unwrap();
    static ref SLUG_INVALID_CHARS_PRETTY: Regex =
        Regex::new(r"([^\p{Alphabetic}0-9\._\~!\$&'\(\)\+,;=@]+)").unwrap();
    static ref SLUG_INVALID_CHARS_ASCII: Regex = Regex::new(r"([^a-zA-Z0-9]+)").unwrap();
}

pub fn slugify(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 0, 1)?;

    let s = input.to_str();
    let mode = if args.is_empty() {
        SlugifyMode::Def
    } else {
        get_mode(&args[0].to_str())
    };

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

fn get_mode(mode_str: &str) -> SlugifyMode {
    match mode_str {
        "none" => SlugifyMode::No,
        "raw" => SlugifyMode::Raw,
        "pretty" => SlugifyMode::Pretty,
        "ascii" => SlugifyMode::Ascii,
        "latin" => SlugifyMode::Latin,
        _ => SlugifyMode::Def,
    }
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
    fn test_slugify_default() {
        assert_eq!(
            unit!(slugify, tos!("The _cönfig.yml file")),
            tos!("the-cönfig-yml-file")
        );
    }

    #[test]
    fn test_slugify_ascii() {
        assert_eq!(
            unit!(slugify, tos!("The _cönfig.yml file"), &[tos!("ascii")]),
            tos!("the-c-nfig-yml-file")
        );
    }

    #[test]
    fn test_slugify_latin() {
        assert_eq!(
            unit!(slugify, tos!("The _cönfig.yml file"), &[tos!("latin")]),
            tos!("the-config-yml-file")
        );
    }

    #[test]
    fn test_slugify_raw() {
        assert_eq!(
            unit!(slugify, tos!("The _config.yml file"), &[tos!("raw")]),
            tos!("the-_config.yml-file")
        );
    }

    #[test]
    fn test_slugify_none() {
        assert_eq!(
            unit!(slugify, tos!("The _config.yml file"), &[tos!("none")]),
            tos!("the _config.yml file")
        );
    }

}
