use std::{convert::TryInto, fmt, ops};

/// Liquid's native date only type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct Date {
    #[serde(with = "friendly_date")]
    pub(crate) inner: DateImpl,
}

type DateImpl = time::Date;

impl Date {
    /// Makes a new NaiveDate from the calendar date (year, month and day).
    ///
    /// Panics on the out-of-range date, invalid month and/or day.
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Self {
        Self {
            inner: DateImpl::from_calendar_date(
                year,
                month.try_into().expect("the month is out of range"),
                day,
            )
            .expect("one or more components were invalid"),
        }
    }

    /// Convert a `str` to `Self`
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(other: &str) -> Option<Self> {
        parse_date(other).map(|d| Self { inner: d })
    }
}

impl Date {
    /// Get the year of the date.
    #[inline]
    pub fn year(&self) -> i32 {
        self.inner.year()
    }
    /// Get the month.
    #[inline]
    pub fn month(&self) -> u8 {
        self.inner.month() as u8
    }
    /// Get the day of the month.
    ///
    //// The returned value will always be in the range 1..=31.
    #[inline]
    pub fn day(&self) -> u8 {
        self.inner.day()
    }
    /// Get the day of the year.
    ///
    /// The returned value will always be in the range 1..=366 (1..=365 for common years).
    #[inline]
    pub fn ordinal(&self) -> u16 {
        self.inner.ordinal()
    }
    /// Get the ISO week number.
    ///
    /// The returned value will always be in the range 1..=53.
    #[inline]
    pub fn iso_week(&self) -> u8 {
        self.inner.iso_week()
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.inner.format(DATE_FORMAT).map_err(|_e| fmt::Error)?
        )
    }
}

impl ops::Deref for Date {
    type Target = DateImpl;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for Date {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

const DATE_FORMAT: &[time::format_description::FormatItem<'_>] =
    time::macros::format_description!("[year]-[month]-[day]");

mod friendly_date {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(date: &DateImpl, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .format(DATE_FORMAT)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateImpl, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateImpl::parse(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
    }
}

fn parse_date(s: &str) -> Option<DateImpl> {
    const USER_FORMATS: &[&[time::format_description::FormatItem<'_>]] = &[
        time::macros::format_description!("[day] [month repr:long] [year]"),
        time::macros::format_description!("[day] [month repr:short] [year]"),
        DATE_FORMAT,
    ];

    match s {
        "today" => Some(time::OffsetDateTime::now_utc().date()),
        _ => USER_FORMATS
            .iter()
            .filter_map(|f| DateImpl::parse(s, f).ok())
            .next(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_date_time_empty_is_bad() {
        let input = "";
        let actual = parse_date(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_time_bad() {
        let input = "aaaaa";
        let actual = parse_date(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_today() {
        let input = "today";
        let actual = parse_date(input);
        assert!(actual.is_some());
    }

    #[test]
    fn parse_long_month() {
        let input = "01 March 2022";
        let actual = parse_date(input);
        assert_eq!(
            DateImpl::from_calendar_date(2022, time::Month::March, 1).unwrap(),
            actual.unwrap()
        );
    }

    #[test]
    fn parse_short_month() {
        let input = "01 Mar 2022";
        let actual = parse_date(input);
        assert_eq!(
            DateImpl::from_calendar_date(2022, time::Month::March, 1).unwrap(),
            actual.unwrap()
        );
    }

    #[test]
    fn parse_iso() {
        let input = "2022-03-02";
        let actual = parse_date(input);
        assert_eq!(
            DateImpl::from_calendar_date(2022, time::Month::March, 2).unwrap(),
            actual.unwrap()
        );
    }
}
