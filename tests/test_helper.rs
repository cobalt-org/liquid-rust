extern crate chrono;
extern crate liquid;
extern crate regex;

pub use liquid::value::Value::Nil;

#[allow(dead_code)]
pub fn render_template<S: AsRef<str>>(
    template: S,
    assigns: &liquid::value::Object,
) -> Result<String, liquid::Error> {
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .parse(template.as_ref())
        .unwrap();
    template.render(assigns)
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! assert_template_result {
    ($expected:expr, $template:expr, ) => {
        assert_template_result!($expected, $template);
    };
    ($expected:expr, $template:expr) => {
        let assigns = ::liquid::value::Value::Object(Default::default());
        assert_template_result!($expected, $template, assigns);
    };
    ($expected:expr, $template:expr, $assigns: expr, ) => {
        assert_template_result!($expected, $template, $assigns);
    };
    ($expected:expr, $template:expr, $assigns: expr) => {
        let rendered = render_template($template, $assigns.as_object().unwrap()).unwrap();
        assert_eq!($expected, rendered);
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! assert_template_matches {
    ($expected:expr, $template:expr, ) => {
        assert_template_matches!($expected, $template);
    };
    ($expected:expr, $template:expr) => {
        let assigns = liquid::value::Value::default();
        assert_template_matches!($expected, $template, assigns);
    };
    ($expected:expr, $template:expr, $assigns: expr, ) => {
        assert_template_matches!($expected, $template, $assigns);
    };
    ($expected:expr, $template:expr, $assigns: expr) => {
        let rendered = render_template($template, $assigns.as_object().unwrap()).unwrap();

        let expected = $expected;
        println!("pattern={}", expected);
        let expected = regex::Regex::new(expected).unwrap();
        println!("rendered={}", rendered);
        assert!(expected.is_match(&rendered));
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! assert_parse_error {
    ($template:expr, ) => {
        assert_parse_error!($template);
    };
    ($template:expr) => {{
        let template = ::liquid::ParserBuilder::with_liquid()
            .build()
            .parse($template);
        assert!(template.is_err());
        template.err().unwrap()
    }};
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! assert_render_error {
    ($template:expr, ) => {
        assert_render_error!($template);
    };
    ($template:expr) => {
        let assigns = ::liquid::value::Value::default();
        assert_render_error!($template, assigns);
    };
    ($template:expr, $assigns: expr, ) => {
        assert_render_error!($template, $assigns);
    };
    ($template:expr, $assigns: expr) => {
        render_template($template, $assigns.as_object().unwrap()).unwrap_err();
    };
}

#[allow(dead_code)]
pub fn date(y: i32, m: u32, d: u32) -> liquid::value::Value {
    use chrono;
    let base = chrono::naive::NaiveDate::from_ymd(y, m, d).and_hms(0, 0, 0);
    let date = liquid::value::Date::from_utc(base, chrono::FixedOffset::east(0));
    liquid::value::Value::scalar(date)
}

#[allow(dead_code)]
pub fn with_time(_time: &str) -> liquid::value::Value {
    Nil
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! v {
    ($($value:tt)+) => {
        value_internal!($($value)+)
    };
}

#[allow(unused_macros)]
macro_rules! value_internal {
    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        value_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        value_internal_vec![$($elems),*]
    };

    // Next element is `nil`.
    (@array [$($elems:expr,)*] nil $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!(nil)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        value_internal!(@array [$($elems,)* value_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        value_internal!(@array [$($elems,)* value_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        value_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        value_unexpected!($unexpected)
    };

    //////////////////////////////////////////////////////////////////////////
    // TT muncher for parsing the inside of an object {...}. Each entry is
    // inserted into the given map variable.
    //
    // Must be invoked as: value_internal!(@object $map () ($($tt)*) ($($tt)*))
    //
    // We require two copies of the input tokens so that we can match on one
    // copy and trigger errors on the other copy.
    //////////////////////////////////////////////////////////////////////////

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert(($($key)+).into(), $value);
        value_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        value_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert(($($key)+).into(), $value);
    };

    // Next value is `nil`.
    (@object $object:ident ($($key:tt)+) (: nil $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!(nil)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        value_internal!(@object $object [$($key)+] (value_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        value_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        value_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        value_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        value_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        value_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: value_internal!($($value)+)
    //////////////////////////////////////////////////////////////////////////

    (nil) => {
        ::liquid::value::Value::Nil
    };

    (true) => {
        ::liquid::value::Value::scalar(true)
    };

    (false) => {
        ::liquid::value::Value::scalar(false)
    };

    ([]) => {
        ::liquid::value::Value::Array(value_internal_vec![])
    };

    ([ $($tt:tt)+ ]) => {
        ::liquid::value::Value::Array(value_internal!(@array [] $($tt)+))
    };

    ({}) => {
        ::liquid::value::Value::Object(Default::default())
    };

    ({ $($tt:tt)+ }) => {
        ::liquid::value::Value::Object({
            let mut object = ::liquid::value::Object::new();
            value_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    ($other:ident) => {
        $other
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        ::liquid::value::to_value($other).unwrap()
    };
}

#[allow(unused_macros)]
macro_rules! value_internal_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! filters {
    ($a:ident, $b:expr) => {{
        filters!($a, $b, )
    }};
    ($a:ident, $b:expr, $($c:expr),*) => {{
        liquid::filters::$a(&$b, &[$($c),*]).unwrap()
    }};
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! filters_fail {
    ($a:ident, $b:expr) => {{
        filters_fail!($a, $b, )
    }};
    ($a:ident, $b:expr, $($c:expr),*) => {{
        liquid::filters::$a(&$b, &[$($c),*]).unwrap_err()
    }};
}
