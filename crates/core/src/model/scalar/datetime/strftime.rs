use std::fmt::{self, Write};

// std::fmt::Write is infallible for String https://doc.rust-lang.org/src/alloc/string.rs.html#2726
// and would only ever fail if we were OOM which Rust won't handle regardless
// so we simplify writes code since we know it can't fail
macro_rules! w {
    ($output:expr, $($arg:tt)*) => {
        $output.write_fmt(format_args!($($arg)*)).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DateFormatError {
    /// A % was not followed by any format specifier
    NoFormatSpecifier,
    /// The pad width could not be parsed
    InvalidWidth(std::num::ParseIntError),
    /// An 'E' or 'O' modifier was encountered and ignored, but there was no
    /// format specifier after it
    NoFormatSpecifierAfterModifier,
}

impl std::error::Error for DateFormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidWidth(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DateFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFormatSpecifier => f.write_str("no format specifier following '%'"),
            Self::NoFormatSpecifierAfterModifier => {
                f.write_str("no format specifier following '%' with a format modifier")
            }
            Self::InvalidWidth(err) => {
                write!(f, "failed to parse padding width: {}", err)
            }
        }
    }
}

/// An implementation of [stftime](https://man7.org/linux/man-pages/man3/strftime.3.html) style formatting.
///
/// Note that in liquid's case we implement the variant
/// [Ruby](https://ruby-doc.org/core-3.0.0/Time.html#method-i-strftime) in
/// particular supports, which may have some deviations from eg C or python etc
///
/// Know exceptions are listed below:
///
/// - `%Z` is used to print the (possibly) abbreviated time zone name. `chrono`
/// did not actually implement this and instead just put the UTC offset with a
/// colon, ie +/-HH:MM, and Ruby itself recommends _not_ using `%Z` as it is
/// OS-dependent on what the string will be, in addition to the abbreviated time
/// zone names being ambiguous. `Z` is also not supported at all by liquidjs.
pub fn strftime(ts: time::OffsetDateTime, fmt: &str) -> Result<String, DateFormatError> {
    let mut output = String::new();
    let mut fmt_iter = fmt.char_indices().peekable();

    while let Some((ind, c)) = fmt_iter.next() {
        if c != '%' {
            output.push(c);
            continue;
        }

        // Keep track of where the '%' was located, if an unknown format specifier
        // is used we backtrack and copy the whole string directly to the output
        let fmt_pos = ind;
        let mut cursor = ind;

        macro_rules! next {
            () => {{
                let next = fmt_iter.next();
                if let Some(nxt) = next {
                    cursor = nxt.0;
                }
                next
            }};
        }

        // Padding is enabled by default, but once it is turned off with `-`
        // it can't be turned on again. Note that the Ruby docs say "don't pad
        // numerical output" but it applies to all format specifiers
        let mut use_padding = true;
        // Numbers are padding with 0 by default, alphabetical by space. At
        // least in the Ruby, the `_` and `0` flags that affect the padding
        // character used can be specified multiple times, but the last one always wins
        let mut padding_style = PaddingStyle::Default;
        // Alphabetical characters will have a default casing eg. "Thu",
        // but can have it changed to uppercase with `^` or inverted with `#`,
        // however note that `#` does not apply to all format specifiers, eg.
        // "Thu" becomes "THU" not "tHU". Like the `_` and `0` flag, the last
        // one wins.
        let mut casing = Casing::Default;

        let (ind, c) = loop {
            match fmt_iter.peek() {
                // whether output is padded or not
                Some((_, '-')) => use_padding = false,
                // use spaces for padding
                Some((_, '_')) => padding_style = PaddingStyle::Space,
                // use zeros for padding
                Some((_, '0')) => padding_style = PaddingStyle::Zero,
                // upcase the result string
                Some((_, '^')) => casing = Casing::Upper,
                // change case
                Some((_, '#')) => casing = Casing::Change,
                None => {
                    return Err(DateFormatError::NoFormatSpecifier);
                }
                // NOTE: Even though in eg. Ruby they say that ':' is a flag,
                // it actually can't come before any width specification, it
                // also doesn't work in conjunction with the (ignored) E/O
                // modifiers so we just parse it as a special case
                Some(next) => break *next,
            }

            next!();
        };

        let padding = if c.is_ascii_digit() {
            loop {
                match fmt_iter.peek() {
                    Some((_, c)) if c.is_ascii_digit() => {
                        next!();
                    }
                    Some((dind, _c)) => {
                        let padding: usize = fmt[ind..*dind]
                            .parse()
                            .map_err(DateFormatError::InvalidWidth)?;

                        break Some(padding);
                    }
                    None => {
                        return Err(DateFormatError::NoFormatSpecifier);
                    }
                }
            }
        } else {
            None
        };

        let (_ind, fmt_char) = {
            let (ind, fmt_char) = next!().ok_or(DateFormatError::NoFormatSpecifier)?;
            // The E and O modifiers are recognized by Ruby, but ignored
            if fmt_char == 'E' || fmt_char == 'O' {
                next!().ok_or(DateFormatError::NoFormatSpecifierAfterModifier)?
            } else {
                (ind, fmt_char)
            }
        };

        enum Formats {
            Numeric(i64, usize),
            Alphabetical(&'static str),
            Formatted,
            Literal(char),
            Unknown,
        }

        let out_cur = output.len();

        macro_rules! write_padding {
            (num $pad_width:expr) => {
                for _ in 0..$pad_width {
                    output.push(match padding_style {
                        PaddingStyle::Default | PaddingStyle::Zero => '0',
                        PaddingStyle::Space => ' ',
                    });
                }
            };
            (comp $pad_width:expr) => {
                if let Some(padding) = padding {
                    for _ in 0..padding.saturating_sub($pad_width) {
                        output.push(match padding_style {
                            PaddingStyle::Default | PaddingStyle::Space => ' ',
                            PaddingStyle::Zero => '0',
                        });
                    }
                }
            };
            ($pad_width:expr) => {
                for _ in 0..$pad_width {
                    output.push(match padding_style {
                        PaddingStyle::Default | PaddingStyle::Space => ' ',
                        PaddingStyle::Zero => '0',
                    });
                }
            };
        }

        let format = match fmt_char {
            // The full proleptic Gregorian year, zero-padded to 4 digits
            'Y' => Formats::Numeric(ts.year() as _, 4),
            // The proleptic Gregorian year divided by 100, zero-padded to 2 digits.
            'C' => Formats::Numeric(ts.year() as i64 / 100, 2),
            // The proleptic Gregorian year modulo 100, zero-padded to 2 digits
            'y' => Formats::Numeric(ts.year() as i64 % 100, 2),
            // Month number (01--12), zero-padded to 2 digits.
            'm' => Formats::Numeric(ts.month() as _, 2),
            // Day number (01--31), zero-padded to 2 digits.
            // Same as %d but space-padded. Same as %_d.
            'd' | 'e' => {
                if fmt_char == 'e' && padding_style == PaddingStyle::Default {
                    padding_style = PaddingStyle::Space;
                }
                Formats::Numeric(ts.day() as _, 2)
            }
            // Sunday = 0, Monday = 1, ..., Saturday = 6.
            'w' => Formats::Numeric(ts.weekday().number_days_from_sunday() as _, 0),
            // Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)
            'u' => Formats::Numeric(ts.weekday().number_from_monday() as _, 0),
            // Week number starting with Sunday (00--53), zero-padded to 2 digits.
            'U' => Formats::Numeric(ts.sunday_based_week() as _, 2),
            // Same as %U, but week 1 starts with the first Monday in that year instead.
            'W' => Formats::Numeric(ts.monday_based_week() as _, 2),
            // Same as %Y but uses the year number in ISO 8601 week date.
            'G' => Formats::Numeric(ts.to_iso_week_date().0 as _, 4),
            // Same as %y but uses the year number in ISO 8601 week date.
            'g' => Formats::Numeric(ts.to_iso_week_date().0 as i64 % 100, 2),
            // Same as %U but uses the week number in ISO 8601 week date (01--53).
            'V' => Formats::Numeric(ts.to_iso_week_date().1 as _, 2),
            // Day of the year (001--366), zero-padded to 3 digits.
            'j' => Formats::Numeric(ts.ordinal() as _, 3),
            // Hour number (00--23), zero-padded to 2 digits.
            // Same as %H but space-padded.
            'H' | 'k' => {
                if fmt_char == 'k' && padding_style == PaddingStyle::Default {
                    padding_style = PaddingStyle::Space;
                }
                Formats::Numeric(ts.hour() as _, 2)
            }
            // Hour number in 12-hour clocks (01--12), zero-padded to 2 digits.
            // OR
            // Same as %I but space-padded.
            'I' | 'l' => {
                let hour = match ts.hour() {
                    0 | 12 => 12,
                    hour @ 1..=11 => hour,
                    over => over - 12,
                };

                if fmt_char == 'l' && padding_style == PaddingStyle::Default {
                    padding_style = PaddingStyle::Space;
                }
                Formats::Numeric(hour as _, 2)
            }
            // Minute number (00--59), zero-padded to 2 digits.
            'M' => Formats::Numeric(ts.minute() as _, 2),
            // Second number (00--60), zero-padded to 2 digits.
            'S' => Formats::Numeric(ts.second() as _, 2),
            // Number of seconds since UNIX_EPOCH
            's' => Formats::Numeric(ts.unix_timestamp(), 0),
            // Abbreviated month name. Always 3 letters.
            'b' | 'h' => Formats::Alphabetical(&(MONTH_NAMES[ts.month() as usize - 1])[..3]),
            // Full month name
            'B' => Formats::Alphabetical(MONTH_NAMES[ts.month() as usize - 1]),
            // Abbreviated weekday name. Always 3 letters.
            'a' => Formats::Alphabetical(&(WEEKDAY_NAMES[ts.weekday() as usize])[..3]),
            // Full weekday name.
            'A' => Formats::Alphabetical(WEEKDAY_NAMES[ts.weekday() as usize]),
            // `am` or `pm` in 12-hour clocks.
            // OR
            // `AM` or `PM` in 12-hour clocks.
            //
            // Note that the case of the result is inverted from the
            // format specifier :bleedingeyes:
            'P' | 'p' => {
                let is_am = ts.hour() < 12;

                let s = if (fmt_char == 'p' && casing != Casing::Change)
                    || (fmt_char == 'P' && casing != Casing::Default)
                {
                    if is_am {
                        "AM"
                    } else {
                        "PM"
                    }
                } else if is_am {
                    "am"
                } else {
                    "pm"
                };

                casing = Casing::Default;
                Formats::Alphabetical(s)
            }
            // Year-month-day format (ISO 8601). Same as %Y-%m-%d.
            'F' => {
                write_padding!(comp 10);
                w!(
                    output,
                    "{:04}-{:02}-{:02}",
                    ts.year(),
                    ts.month() as u8,
                    ts.day(),
                );
                Formats::Formatted
            }
            // Day-month-year format. Same as %e-%^b-%Y.
            'v' => {
                // special case where the month is always uppercased
                casing = Casing::Upper;
                write_padding!(comp 11);
                w!(
                    output,
                    "{:>2}-{}-{:04}",
                    ts.day(),
                    &(MONTH_NAMES[ts.month() as usize - 1])[..3],
                    ts.year(),
                );
                Formats::Formatted
            }
            // Hour-minute format. Same as %H:%M.
            'R' => {
                write_padding!(comp 5);
                w!(output, "{:02}:{:02}", ts.hour(), ts.minute());
                Formats::Formatted
            }
            // Month-day-year format. Same as %m/%d/%y
            'D' | 'x' => {
                write_padding!(comp 8);
                w!(
                    output,
                    "{:02}/{:02}/{:02}",
                    ts.month() as u8,
                    ts.day(),
                    ts.year() % 100,
                );
                Formats::Formatted
            }
            // Hour-minute-second format. Same as %H:%M:%S.
            'T' | 'X' => {
                write_padding!(comp 8);
                w!(
                    output,
                    "{:02}:{:02}:{:02}",
                    ts.hour(),
                    ts.minute(),
                    ts.second()
                );
                Formats::Formatted
            }
            // Hour-minute-second format in 12-hour clocks. Same as %I:%M:%S %p.
            'r' => {
                let hour = match ts.hour() {
                    0 | 12 => 12,
                    hour @ 1..=11 => hour,
                    over => over - 12,
                };

                let is_am = ts.hour() < 12;

                write_padding!(comp 11);
                w!(
                    output,
                    "{:02}:{:02}:{:02} {}",
                    hour,
                    ts.minute(),
                    ts.second(),
                    if is_am { "AM" } else { "PM" },
                );
                Formats::Formatted
            }
            // Date and time. Same as %a %b %e %T %Y
            'c' => {
                write_padding!(comp 24);
                w!(
                    output,
                    "{} {} {:>2} {:02}:{:02}:{:02} {:04}",
                    &(WEEKDAY_NAMES[ts.weekday() as usize])[..3],
                    &(MONTH_NAMES[ts.month() as usize - 1])[..3],
                    ts.day(),
                    ts.hour(),
                    ts.minute(),
                    ts.second(),
                    ts.year()
                );
                Formats::Formatted
            }
            // Literals
            '%' => Formats::Literal('%'),
            'n' => Formats::Literal('\n'),
            't' => Formats::Literal('\t'),
            // L
            // Millisecond of the second (000..999) this one was not supported by chrono
            // N
            // Fractional seconds digits, default is 9 digits (nanosecond)
            // For some reason Ruby says it supports printing out pico to yocto
            // seconds but we only have nanosecond precision, and I think they probably
            // do as well, so we just well, pretend if the user gives us something
            // that ridiculous
            //
            // Note that this is a special case where the padding width that applies
            // to other specifiers actually means the number of digits to print, and
            // any digits above (in our case) nanosecond are always to the right and
            // always 0, not spaces, so the normal format specifiers are ignored
            'L' | 'N' => {
                let nanos = ts.nanosecond();
                let digits = padding.unwrap_or(if fmt_char == 'L' { 3 } else { 9 });

                w!(
                    output,
                    "{:0<width$}",
                    if digits <= 9 {
                        nanos / 10u32.pow(9 - digits as u32)
                    } else {
                        nanos
                    },
                    width = digits
                );

                continue;
            }
            // %z - Time zone as hour and minute offset from UTC (e.g. +0900)
            // %:z - hour and minute offset from UTC with a colon (e.g. +09:00)
            // %::z - hour, minute and second offset from UTC (e.g. +09:00:00)
            'z' | 'Z' | ':' => {
                // So Ruby _supposedly_ outputs the (OS dependent) time zone name/abbreviation
                // however in my testing Z was instead completely ignored. In this
                // case we preserve the previous chrono behavior of just output +/-HH:MM
                let hm_sep = matches!(fmt_char, 'Z' | ':');
                let mut ms_sep = false;

                let mut handle_colons = || {
                    if fmt_char == ':' {
                        match next!() {
                            Some((_, 'z')) => {
                                return true;
                            }
                            Some((_, ':')) => {
                                if let Some((_, 'z')) = next!() {
                                    ms_sep = true;
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                            _ => return false,
                        }
                    }

                    true
                };

                if handle_colons() {
                    let offset = ts.offset();

                    // The timezone padding is calculated by the total size of the
                    // output, but for rust fmt strings it only applies to the hour
                    // component
                    let output_size = 1 // +/-
                        + 2 // HH
                        + if hm_sep { 1 } else { 0 } // :
                        + 2 // MM
                        + if ms_sep {
                            1 + 2 // :ss
                        } else {
                            0
                        };

                    // Note that z doesn't respect `-` even if it is numeric, mostly
                    let pad_width = std::cmp::max(
                        padding.unwrap_or_default().saturating_sub(output_size) + 2,
                        2,
                    );

                    if padding_style != PaddingStyle::Space {
                        // So 0 filling to the left with a sign doesn't do at all
                        // what you would expect, eg +0600 becomes 0+600, so we
                        // do it manually

                        w!(
                            output,
                            "{}{:0>width$}",
                            if offset.is_negative() { '-' } else { '+' },
                            offset.whole_hours().abs(),
                            width = pad_width,
                        );
                    } else {
                        w!(
                            output,
                            "{: >+width$}",
                            offset.whole_hours(),
                            width = pad_width
                        );
                    }

                    w!(
                        output,
                        "{}{:02}",
                        if hm_sep { ":" } else { "" },
                        offset.minutes_past_hour().abs()
                    );

                    if ms_sep {
                        w!(output, ":{:02}", offset.seconds_past_minute().abs());
                    }

                    continue;
                }

                Formats::Unknown
            }
            // Unknown format specifier
            _ => Formats::Unknown,
        };

        match format {
            Formats::Numeric(value, def_padding) => {
                if use_padding {
                    let mut digits = match value {
                        0 => 1,
                        neg if neg < 0 => 1,
                        _ => 0,
                    };
                    let mut v = value;

                    while v != 0 {
                        v /= 10;
                        digits += 1;
                    }

                    if value < 0 && padding_style != PaddingStyle::Space {
                        output.push('-');
                    }

                    write_padding!(num padding.unwrap_or(def_padding + if value < 0 { 1 } else { 0 }).saturating_sub(digits));

                    if value < 0 && padding_style == PaddingStyle::Space {
                        output.push('-');
                    }
                } else if value < 0 {
                    output.push('-');
                }

                w!(output, "{}", value.abs());
            }
            Formats::Alphabetical(s) => {
                if use_padding && padding.is_some() {
                    write_padding!(padding.unwrap_or_default().saturating_sub(s.len()));
                }
                output.push_str(s);
                if casing != Casing::Default {
                    output[out_cur..].make_ascii_uppercase();
                }
            }
            Formats::Formatted => {
                if casing != Casing::Default {
                    output[out_cur..].make_ascii_uppercase();
                }
            }
            Formats::Literal(lit) => {
                if use_padding && padding.is_some() {
                    write_padding!(padding.unwrap_or_default().saturating_sub(1));
                }
                output.push(lit);
            }
            Formats::Unknown => {
                output.push_str(&fmt[fmt_pos..=cursor]);
                continue;
            }
        };
    }

    Ok(output)
}

#[derive(Copy, Clone, PartialEq)]
enum PaddingStyle {
    /// Use 0 for numeric outputs and spaces for alphabetical ones
    Default,
    /// Use 0 for padding
    Zero,
    /// Use space for padding
    Space,
}

#[derive(Copy, Clone, PartialEq)]
enum Casing {
    /// Alphabetical characters should be outputted per their defaults
    Default,
    /// The `^` flag has been used, so all ascii alphabetical characters should be uppercase
    Upper,
    /// The `#` flag has been used, so all ascii alphabetical characters should have their case changed
    Change,
}

// time unfortunately only implements `Display` for Month and hides
// its more sophisticated formatting internally, so we just have our own table
const MONTH_NAMES: &[&str] = &[
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// Ditto
const WEEKDAY_NAMES: &[&str] = &[
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

#[cfg(test)]
mod test {
    use super::*;

    const SIMPLE: time::OffsetDateTime =
        time::macros::datetime!(2022-11-03 07:56:37.666_777_888 +06:00);

    macro_rules! eq {
        ($ts:expr => [$($fmt:expr => $exp:expr),+$(,)?]) => {
            $(
                match strftime($ts, $fmt) {
                    Ok(formatted) => {
                        assert_eq!(formatted, $exp, "format string '{}' gave unexpected results", stringify!($fmt));
                    }
                    Err(err) => {
                        panic!("failed to format with '{}': {}", stringify!($fmt), err);
                    }
                }
            )+
        };
    }

    #[test]
    fn basic() {
        eq!(SIMPLE => [
            // year
            "%Y" => "2022",
            "%C" => "20",
            "%y" => "22",

            // month
            "%m" => "11",
            "%B" => "November",
            "%b" => "Nov",
            "%h" => "Nov",

            // day
            "%d" => "03",
            "%e" => " 3",
            "%j" => "307",

            // time
            "%H" => "07",
            "%k" => " 7",
            "%I" => "07",
            "%l" => " 7",

            "%P" => "am",
            "%p" => "AM",

            "%M" => "56",
            "%S" => "37",
            "%L" => "666",

            "%N" => "666777888",
            "%1N" => "6",
            "%3N" => "666",
            "%6N" => "666777",
            "%9N" => "666777888",
            "%12N" => "666777888000",
            "%24N" => "666777888000000000000000",

            // timezone
            "%z" => "+0600",
            "%Z" => "+06:00",

            // weekday
            "%A" => "Thursday",
            "%a" => "Thu",
            "%u" => "4",
            "%w" => "4",

            // ISO
            "%G" => "2022",
            "%g" => "22",
            "%V" => "44",

            // Week number
            "%U" => "44",
            "%W" => "44",

            // UNIX timestamp
            "%s" => "1667440597",

            // Literals
            "%n" => "\n",
            "%t" => "\t",
            "%%" => "%",

            // Composites
            "%c" => "Thu Nov  3 07:56:37 2022",
            "%D" => "11/03/22",
            "%F" => "2022-11-03",
            "%v" => " 3-NOV-2022",
            "%x" => "11/03/22",
            "%T" => "07:56:37",
            "%X" => "07:56:37",
            "%r" => "07:56:37 AM",
            "%R" => "07:56",
        ]);
    }

    /// ISO composite formats taken directly from https://ruby-doc.org/core-3.0.0/Time.html#method-i-strftime
    #[test]
    fn iso_composites() {
        eq!(time::macros::datetime!(2007-11-19 08:37:48 -06:00) => [
            "%Y%m%d"            => "20071119",                  // Calendar date (basic)
            "%F"                => "2007-11-19",                // Calendar date (extended)
            "%Y-%m"             => "2007-11",                   // Calendar date, reduced accuracy, specific month
            "%Y"                => "2007",                      // Calendar date, reduced accuracy, specific year
            "%C"                => "20",                        // Calendar date, reduced accuracy, specific century
            "%Y%j"              => "2007323",                   // Ordinal date (basic)
            "%Y-%j"             => "2007-323",                  // Ordinal date (extended)
            "%GW%V%u"           => "2007W471",                  // Week date (basic)
            "%G-W%V-%u"         => "2007-W47-1",                // Week date (extended)
            "%GW%V"             => "2007W47",                   // Week date, reduced accuracy, specific week (basic)
            "%G-W%V"            => "2007-W47",                  // Week date, reduced accuracy, specific week (extended)
            "%H%M%S"            => "083748",                    // Local time (basic)
            "%T"                => "08:37:48",                  // Local time (extended)
            "%H%M"              => "0837",                      // Local time, reduced accuracy, specific minute (basic)
            "%H:%M"             => "08:37",                     // Local time, reduced accuracy, specific minute (extended)
            "%H"                => "08",                        // Local time, reduced accuracy, specific hour
            "%H%M%S,%L"         => "083748,000",                // Local time with decimal fraction, comma as decimal sign (basic)
            "%T,%L"             => "08:37:48,000",              // Local time with decimal fraction, comma as decimal sign (extended)
            "%H%M%S.%L"         => "083748.000",                // Local time with decimal fraction, full stop as decimal sign (basic)
            "%T.%L"             => "08:37:48.000",              // Local time with decimal fraction, full stop as decimal sign (extended)
            "%H%M%S%z"          => "083748-0600",               // Local time and the difference from UTC (basic)
            "%T%:z"             => "08:37:48-06:00",            // Local time and the difference from UTC (extended)
            "%Y%m%dT%H%M%S%z"   => "20071119T083748-0600",      // Date and time of day for calendar date (basic)
            "%FT%T%:z"          => "2007-11-19T08:37:48-06:00", // Date and time of day for calendar date (extended)
            "%Y%jT%H%M%S%z"     => "2007323T083748-0600",       // Date and time of day for ordinal date (basic)
            "%Y-%jT%T%:z"       => "2007-323T08:37:48-06:00",   // Date and time of day for ordinal date (extended)
            "%GW%V%uT%H%M%S%z"  => "2007W471T083748-0600",      // Date and time of day for week date (basic)
            "%G-W%V-%uT%T%:z"   => "2007-W47-1T08:37:48-06:00", // Date and time of day for week date (extended)
            "%Y%m%dT%H%M"       => "20071119T0837",             // Calendar date and local time (basic)
            "%FT%R"             => "2007-11-19T08:37",          // Calendar date and local time (extended)
            "%Y%jT%H%MZ"        => "2007323T0837Z",             // Ordinal date and UTC of day (basic)
            "%Y-%jT%RZ"         => "2007-323T08:37Z",           // Ordinal date and UTC of day (extended)
            "%GW%V%uT%H%M%z"    => "2007W471T0837-0600",        // Week date and local time and difference from UTC (basic)
            "%G-W%V-%uT%R%:z"   => "2007-W47-1T08:37-06:00",    // Week date and local time and difference from UTC (extended)
        ]);
    }

    #[test]
    fn upper_flag() {
        eq!(time::macros::datetime!(2007-01-19 08:37:48 -06:00) => [
            "%^b" => "JAN",
            "%^h" => "JAN",
            "%^B" => "JANUARY",
            "%^a" => "FRI",
            "%^A" => "FRIDAY",
            "%^p" => "AM",
            "%^P" => "AM",
            "%^v" => "19-JAN-2007",
        ]);
    }

    #[test]
    fn change_flag() {
        eq!(time::macros::datetime!(2007-12-19 18:37:48 +08:00) => [
            "%#b" => "DEC",
            "%#h" => "DEC",
            "%#B" => "DECEMBER",
            "%#a" => "WED",
            "%#A" => "WEDNESDAY",
            "%#p" => "pm",
            "%#P" => "PM",
            "%#v" => "19-DEC-2007",
        ]);
    }

    #[test]
    fn padding() {
        eq!(time::macros::datetime!(2022-01-03 07:56:37.666_777_888 +06:00) => [
            // year
            "%8Y" => "00002022",
            "%_11Y" => "       2022",
            "%1C" => "20",
            "%2C" => "20",
            "%3C" => "020",
            "%_4C" => "  20",
            "%_5y" => "   22",
            "%-_5y" => "22",
            "%_-5y" => "22",
            "%-5y" => "22",

            // month
            "%13m" => "0000000000001",
            "%_13m" => "            1",
            "%7B" => "January",
            "%8B" => " January",
            "%_8B" => " January",
            "%08B" => "0January",
            "%7b" => "    Jan",
            "%07h" => "0000Jan",
            "%-07h" => "Jan",

            // day
            "%2d" => "03",
            "%3d" => "003",
            "%_5d" => "    3",
            "%3e" => "  3",
            "%_3e" => "  3",
            "%03e" => "003",
            "%j" => "003",
            "%_j" => "  3",
            "%1j" => "3",
            "%2j" => "03",
            "%_2j" => " 3",

            // time
            "%_H" => " 7",
            "%0k" => "07",
            "%_I" => " 7",
            "%04l" => "0007",

            "%4P" => "  am",
            "%04p" => "00AM",
            "%01p" => "AM",

            "%9M" => "000000056",
            "%_10S" => "        37",
            "%_20L" => "66677788800000000000",
            "%-_20L" => "66677788800000000000",

            "%_N" => "666777888",
            "%_1N" => "6",
            "%_3N" => "666",
            "%_6N" => "666777",
            "%_9N" => "666777888",
            "%_12N" => "666777888000",
            "%-_24N" => "666777888000000000000000",

            // timezone
            "%1z" => "+0600",
            "%2z" => "+0600",
            "%3z" => "+0600",
            "%4z" => "+0600",
            "%5z" => "+0600",
            "%6z" => "+00600",
            "%10z" => "+000000600",
            "%10Z" => "+000006:00",
            "%10::z" => "+006:00:00",

            // weekday
            "%4A" => "Monday",
            "%10A" => "    Monday",
            "%10a" => "       Mon",
            "%-10a" => "Mon",
            "%04a" => "0Mon",
            "%05u" => "00001",
            "%_5w" => "    1",

            // ISO
            "%13G" => "0000000002022",
            "%_13g" => "           22",
            "%V" => "01",

            // Week number
            "%U" => "01",
            "%W" => "01",

            // UNIX timestamp
            "%10s" => "1641174997",
            "%20s" => "00000000001641174997",

            // Literals
            "%2n" => " \n",
            "%05t" => "0000\t",
            "%10%" => "         %",

            // Composites
            "%30c" => "      Mon Jan  3 07:56:37 2022",
            "%8D" => "01/03/22",
            "%012F" => "002022-01-03",
            "%012v" => "0 3-JAN-2022",
            "%3x" => "01/03/22",
            "%11T" => "   07:56:37",
            "%8X" => "07:56:37",
            "%10X" => "  07:56:37",
            "%010X" => "0007:56:37",
            "%12r" => " 07:56:37 AM",
            "%-6R" => " 07:56",
        ]);

        eq!(time::macros::datetime!(-20-06-13 17:56:37.666_777_888 -07:25) => [
            "%Y" => "-0020",
            "%_Y" => "  -20",
            "%4Y" => "-020",
            "%_4Y" => " -20",

            "%1z" => "-0725",
            "%2z" => "-0725",
            "%3z" => "-0725",
            "%4z" => "-0725",
            "%5z" => "-0725",
            "%6z" => "-00725",
            "%10z" => "-000000725",
            "%10Z" => "-000007:25",
            "%10::z" => "-007:25:00",
        ]);
    }

    #[test]
    fn handles_unknown() {
        eq!(time::macros::datetime!(-20-06-13 17:56:37.666_777_888 -06:00) => [
            "%:b" => "%:b",
            "%-_::xX%Y" => "%-_::xX-0020",
            "%-_::xX%4Y" => "%-_::xX-020",
            "%_0-^#^q" => "%_0-^#^q",
        ]);
    }

    #[test]
    fn errors() {
        assert_eq!(
            strftime(SIMPLE, "%9").unwrap_err(),
            DateFormatError::NoFormatSpecifier
        );
        assert_eq!(
            strftime(SIMPLE, "%9E").unwrap_err(),
            DateFormatError::NoFormatSpecifierAfterModifier
        );
        assert_eq!(
            strftime(SIMPLE, "%010").unwrap_err(),
            DateFormatError::NoFormatSpecifier
        );
        assert_eq!(
            strftime(SIMPLE, "X%").unwrap_err(),
            DateFormatError::NoFormatSpecifier
        );
        assert!(matches!(
            strftime(SIMPLE, "%18446744073709551616d").unwrap_err(),
            DateFormatError::InvalidWidth(_)
        ));
    }
}
