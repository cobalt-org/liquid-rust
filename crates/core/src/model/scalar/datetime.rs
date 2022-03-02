use std::{fmt, ops};

use super::Date;

type DateTimeImpl = time::OffsetDateTime;

/// Liquid's native date + time type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DateTime {
    #[serde(with = "friendly_date_time")]
    inner: DateTimeImpl,
}

impl DateTime {
    /// Create a `DateTime` from the current moment.
    pub fn now() -> Self {
        Self {
            inner: DateTimeImpl::now_utc(),
        }
    }

    /// Convert a `str` to `Self`
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(other: &str) -> Option<Self> {
        parse_date_time(other).map(|d| Self { inner: d })
    }

    /// Replace date with `other`.
    pub fn with_date(self, other: Date) -> Self {
        Self {
            inner: self.inner.replace_date(other.inner),
        }
    }

    /// Changes the associated time zone. This does not change the actual DateTime (but will change the string representation).
    pub fn with_offset(self, offset: time::UtcOffset) -> Self {
        Self {
            inner: self.inner.to_offset(offset),
        }
    }

    /// Retrieves a date component.
    pub fn date(self) -> Date {
        Date {
            inner: self.inner.date(),
        }
    }

    /// Formats the combined date and time with the specified format string.
    ///
    /// See the [chrono::format::strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
    /// module on the supported escape sequences.
    pub fn format(&self, fmt: &str) -> Result<String, fmt::Error> {
        use std::fmt::Write;
        let mut s = String::new();

        let mut cursor = 0;
        let ts = self.inner;

        while let Some(ind) = fmt[cursor..].find('%') {
            // Handle invalid case of a '%' at the end of the format string
            if ind == fmt.len() {
                return Err(fmt::Error);
            }

            if ind > 0 {
                s.push_str(&fmt[cursor..cursor + ind]);
            }

            cursor = cursor + ind + 1;

            let fmt_char = &fmt[cursor..cursor + 1];

            match fmt_char {
                // START: DATE SPECIFIERS
                // The full proleptic Gregorian year, zero-padded to 4 digits
                "Y" => {
                    write!(&mut s, "{:04}", ts.year())?;
                }
                // The proleptic Gregorian year divided by 100, zero-padded to 2 digits.
                "C" => {
                    write!(&mut s, "{:02}", ts.year() / 100)?;
                }
                // The proleptic Gregorian year modulo 100, zero-padded to 2 digits
                "y" => {
                    write!(&mut s, "{:02}", ts.year() % 100)?;
                }
                // Month number (01--12), zero-padded to 2 digits.
                "m" => {
                    write!(&mut s, "{:02}", ts.month() as u8)?;
                }
                // Abbreviated month name. Always 3 letters.
                "b" | "h" => {
                    s.push_str(&(MONTH_NAMES[ts.month() as usize - 1])[..3]);
                }
                // Full month name
                "B" => {
                    s.push_str(MONTH_NAMES[ts.month() as usize - 1]);
                }
                // Day number (01--31), zero-padded to 2 digits.
                "d" => {
                    write!(&mut s, "{:02}", ts.day())?;
                }
                // Same as %d but space-padded. Same as %_d.
                "e" => {
                    write!(&mut s, "{:>2}", ts.day())?;
                }
                // Abbreviated weekday name. Always 3 letters.
                "a" => {
                    s.push_str(&(WEEKDAY_NAMES[ts.weekday() as usize])[..3]);
                }
                // Full weekday name.
                "A" => {
                    s.push_str(WEEKDAY_NAMES[ts.weekday() as usize]);
                }
                // Sunday = 0, Monday = 1, ..., Saturday = 6.
                "w" => {
                    write!(&mut s, "{}", ts.weekday().number_days_from_sunday())?;
                }
                // Monday = 1, Tuesday = 2, ..., Sunday = 7. (ISO 8601)
                "u" => {
                    write!(&mut s, "{}", ts.weekday().number_from_monday())?;
                }
                // Week number starting with Sunday (00--53), zero-padded to 2 digits.
                "U" => {
                    write!(&mut s, "{:02}", ts.sunday_based_week())?;
                }
                // Same as %U, but week 1 starts with the first Monday in that year instead.
                "W" => {
                    write!(&mut s, "{:02}", ts.monday_based_week())?;
                }
                // Same as %Y but uses the year number in ISO 8601 week date.
                "G" => {
                    write!(&mut s, "{:04}", ts.to_iso_week_date().0)?;
                }
                // Same as %y but uses the year number in ISO 8601 week date.
                "g" => {
                    write!(&mut s, "{:02}", ts.to_iso_week_date().0 % 100)?;
                }
                // Same as %U but uses the week number in ISO 8601 week date (01--53).
                "V" => {
                    write!(&mut s, "{:02}", ts.to_iso_week_date().1)?;
                }
                // Day of the year (001--366), zero-padded to 3 digits.
                "j" => {
                    write!(&mut s, "{:03}", ts.ordinal())?;
                }
                // Month-day-year format. Same as %m/%d/%y
                //
                // 'x' - Locale's date representation (e.g., 12/31/99). In chrono
                // this would default to the same as D, and only actually do
                // locale aware formatting if the `unstable-locales` feature
                // was enabled, which this crate did _not_ enable, so using
                // the same formatting here makes this work the same as with chrono
                "D" | "x" => {
                    write!(
                        &mut s,
                        "{:02}/{:02}/{:02}",
                        ts.month() as u8,
                        ts.day(),
                        ts.year() % 100,
                    )?;
                }
                // Year-month-day format (ISO 8601). Same as %Y-%m-%d.
                "F" => {
                    write!(
                        &mut s,
                        "{:04}/{:02}/{:02}",
                        ts.year(),
                        ts.month() as u8,
                        ts.day(),
                    )?;
                }
                // Day-month-year format. Same as %e-%b-%Y.
                "v" => {
                    write!(
                        &mut s,
                        "{:>2}/{:02}/{:04}",
                        ts.day(),
                        &(MONTH_NAMES[ts.month() as usize - 1])[..3],
                        ts.year(),
                    )?;
                }
                // END: DATE SPECIFIERS
                // START: TIME SPECIFIERS
                // Hour number (00--23), zero-padded to 2 digits.
                "H" => {
                    write!(&mut s, "{:02}", ts.hour())?;
                }
                // Same as %H but space-padded. Same as %_H.
                "k" => {
                    write!(&mut s, "{:>2}", ts.hour())?;
                }
                // Hour number in 12-hour clocks (01--12), zero-padded to 2 digits.
                // OR
                // Same as %I but space-padded. Same as %_I.
                "I" | "l" => {
                    let hour = match ts.hour() {
                        0 | 12 => 12,
                        hour @ 1..=11 => hour,
                        over => over - 12,
                    };

                    if fmt_char == "I" {
                        write!(&mut s, "{:02}", hour,)?;
                    } else {
                        write!(&mut s, "{:>2}", hour,)?;
                    }
                }
                // `am` or `pm` in 12-hour clocks.
                // OR
                // `AM` or `PM` in 12-hour clocks.
                //
                // Note that the case of the result is inverted from the
                // format specifier :bleedingeyes:
                "P" | "p" => {
                    let is_am = ts.hour() < 12;

                    if fmt_char == "P" {
                        s.push_str(if is_am { "am" } else { "pm" });
                    } else {
                        s.push_str(if is_am { "AM" } else { "PM" });
                    }
                }
                // Minute number (00--59), zero-padded to 2 digits.
                "M" => {
                    write!(&mut s, "{:02}", ts.minute())?;
                }
                // Second number (00--60), zero-padded to 2 digits.
                "S" => {
                    write!(&mut s, "{:02}", ts.second())?;
                }
                // This is not actually supported by chrono/strftime, but is
                // the liquid/Ruby way to get the milliseconds, zero-padded to 3 digits
                "L" => {
                    write!(&mut s, "{:03}", ts.millisecond())?;
                }
                // Hour-minute format. Same as %H:%M.
                "R" => {
                    write!(&mut s, "{:02}:{:02}", ts.hour(), ts.minute())?;
                }
                // Hour-minute-second format. Same as %H:%M:%S.
                "T" | "X" => {
                    write!(
                        &mut s,
                        "{:02}:{:02}:{:02}",
                        ts.hour(),
                        ts.minute(),
                        ts.second()
                    )?;
                }
                // Hour-minute-second format in 12-hour clocks. Same as %I:%M:%S %p.
                "r" => {
                    let hour = match ts.hour() {
                        0 | 12 => 12,
                        hour @ 1..=11 => hour,
                        over => over - 12,
                    };

                    let is_am = ts.hour() < 12;

                    write!(
                        &mut s,
                        "{:02}:{:02}:{:02} {}",
                        hour,
                        ts.minute(),
                        ts.second(),
                        if is_am { "AM" } else { "PM" },
                    )?;
                }
                // Offset from the local time to UTC (with UTC being +0000).
                // OR
                // `Z` is supposedly supported by chrono according to the
                // docs, used to print the time zone name/abbreviation, but
                // in reality it is not, and instead just prints the
                // same as `z` but with the components delimited with ':'
                "Z" | "z" => {
                    let offset = ts.offset();

                    write!(
                        &mut s,
                        "{}{:02}{}{:02}",
                        if offset.is_negative() { "-" } else { "+" },
                        offset.whole_hours().abs(),
                        if fmt_char == "Z" { ":" } else { "" },
                        offset.minutes_past_hour().abs()
                    )?;
                }
                // END: TIME SPECIFIERS
                // START: DATETIME SPECIFIERS
                // Locale's date and time (e.g., Thu Mar 3 23:05:25 2005).
                //
                // As stated before, chrono doesn't truly support this due
                // to needing actual locale information, so we again punt
                "c" => {
                    write!(
                        &mut s,
                        "{} {} {} {:02}:{:02}:{:02} {:04}",
                        &(WEEKDAY_NAMES[ts.weekday() as usize])[..3],
                        &(MONTH_NAMES[ts.month() as usize - 1])[..3],
                        ts.day(),
                        ts.hour(),
                        ts.minute(),
                        ts.second(),
                        ts.year()
                    )?;
                }
                // RFC-3339. I'm not actually sure if this is supported by
                // liquid.
                "+" => {
                    let offset = ts.offset();
                    write!(
                        &mut s,
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}{:02}:{:02}",
                        ts.year(),
                        ts.month() as u8,
                        ts.day(),
                        ts.hour(),
                        ts.minute(),
                        ts.second(),
                        if offset.is_negative() { "-" } else { "+" },
                        offset.whole_hours().abs(),
                        offset.minutes_past_hour().abs(),
                    )?;
                }
                // UNIX timestamp, the number of seconds since 1970-01-01 00:00 UTC.
                "s" => {
                    write!(&mut s, "{}", ts.unix_timestamp())?;
                }
                // END: DATETIME SPECIFIERS
                // Tab
                "t" => s.push('\t'),
                // Newline
                "n" => s.push('\n'),
                // Literal %
                "%" => s.push('%'),
                // Format specifiers that liquid/Ruby can use that we can't
                _ => return Err(fmt::Error),
            }

            cursor += 1;
        }

        if cursor < fmt.len() {
            s.push_str(&fmt[cursor..]);
        }

        Ok(s)
    }
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

impl Default for DateTime {
    fn default() -> Self {
        Self {
            inner: DateTimeImpl::UNIX_EPOCH,
        }
    }
}

const DATE_TIME_FORMAT: &[time::format_description::FormatItem<'static>] = time::macros::format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.inner
                .format(DATE_TIME_FORMAT)
                .map_err(|_e| fmt::Error)?
        )
    }
}

impl ops::Deref for DateTime {
    type Target = DateTimeImpl;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for DateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

mod friendly_date_time {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(date: &DateTimeImpl, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .format(DATE_TIME_FORMAT)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeImpl, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: std::borrow::Cow<'_, str> = Deserialize::deserialize(deserializer)?;
        DateTimeImpl::parse(&s, DATE_TIME_FORMAT).map_err(serde::de::Error::custom)
    }
}

fn parse_date_time(s: &str) -> Option<DateTimeImpl> {
    const USER_FORMATS: &[&[time::format_description::FormatItem<'_>]] = &[
        time::macros::format_description!("[day] [month repr:long] [year] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"),
        time::macros::format_description!("[day] [month repr:short] [year] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"),
        DATE_TIME_FORMAT,
    ];

    match s {
        "now" => Some(DateTimeImpl::now_utc()),
        _ => USER_FORMATS
            .iter()
            .filter_map(|f| DateTimeImpl::parse(s, f).ok())
            .next(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_date_time_empty_is_bad() {
        let input = "";
        let actual = parse_date_time(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_time_bad() {
        let input = "aaaaa";
        let actual = parse_date_time(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_time_now() {
        let input = "now";
        let actual = parse_date_time(input);
        assert!(actual.is_some());
    }

    #[test]
    fn parse_date_time_serialized_format() {
        let input = "2016-02-16 10:00:00 +0100";
        let actual = parse_date_time(input);
        assert!(actual.is_some());
    }

    #[test]
    fn parse_date_time_to_string() {
        let date = DateTime::now();
        let input = date.to_string();
        let actual = parse_date_time(&input);
        assert!(actual.is_some());
    }
}
