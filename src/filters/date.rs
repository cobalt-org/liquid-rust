#[cfg(feature = "extra-filters")]
use chrono::FixedOffset;

use value::Value;
use value::Scalar;
use interpreter::{FilterError, FilterResult};

use super::check_args_len;

pub fn date(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 1, 0)?;

    let format = args[0].to_str();
    if format.is_empty() {
        return Ok(input.clone());
    }

    let date = input.as_scalar().and_then(Scalar::to_date);
    let date = match date {
        Some(d) => d,
        None => {
            return Ok(input.clone());
        }
    };

    Ok(Value::scalar(date.format(format.as_ref()).to_string()))
}

// liquid-rust proprietary

#[cfg(feature = "extra-filters")]
pub fn date_in_tz(input: &Value, args: &[Value]) -> FilterResult {
    check_args_len(args, 2, 0)?;

    let date = input
        .as_scalar()
        .and_then(Scalar::to_date)
        .ok_or(FilterError::InvalidType("Invalid date format".into()))?;

    let format = args[0].to_str();

    let n =
        args[1]
            .as_scalar()
            .and_then(Scalar::to_integer)
            .ok_or_else(|| FilterError::InvalidArgument(1, "Whole number expected".to_owned()))?;
    let timezone = FixedOffset::east(n * 3600);

    Ok(Value::scalar(date.with_timezone(&timezone)
                         .format(format.as_ref())
                         .to_string()))
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! unit {
        ( $a:ident, $b:expr ) => {{
            unit!($a, $b, &[])
        }};
        ( $a:ident, $b:expr , $c:expr) => {{
            $a(&$b, $c).unwrap()
        }};
    }

    macro_rules! failed {
        ( $a:ident, $b:expr ) => {{
            failed!($a, $b, &[])
        }};
        ( $a:ident, $b:expr, $c:expr ) => {{
            $a(&$b, $c).unwrap_err()
        }};
    }

    macro_rules! tos {
        ( $a:expr ) => {{
            Value::scalar($a.to_owned())
        }};
    }

    #[test]
    fn unit_date() {
        assert_eq!(unit!(date,
                         tos!("13 Jun 2016 02:30:00 +0300"),
                         &[tos!("%Y-%m-%d")]),
                   tos!("2016-06-13"));
    }

    #[test]
    fn unit_date_cobalt_format() {
        assert_eq!(unit!(date, tos!("2016-06-13 02:30:00 +0300"), &[tos!("%Y-%m-%d")]),
                   tos!("2016-06-13"));
    }

    #[test]
    fn unit_date_bad_input_type() {
        assert_eq!(unit!(date, Value::scalar(0f32), &[tos!("%Y-%m-%d")]),
                   Value::scalar(0f32));
    }

    #[test]
    fn unit_date_bad_input_format() {
        assert_eq!(unit!(date, tos!("blah blah blah"), &[tos!("%Y-%m-%d")]),
                   tos!("blah blah blah"));
    }

    #[test]
    fn unit_date_format_empty() {
        assert_eq!(unit!(date,
                         tos!("13 Jun 2016 02:30:00 +0300"),
                         &[Value::scalar("".to_owned())]),
                   tos!("13 Jun 2016 02:30:00 +0300"));
    }

    #[test]
    fn unit_date_bad_format_type() {
        assert_eq!(unit!(date,
                         tos!("13 Jun 2016 02:30:00 +0300"),
                         &[Value::scalar(0f32)]),
                   tos!("0"));
    }

    #[test]
    fn unit_date_missing_format() {
        assert_eq!(failed!(date, tos!("13 Jun 2016 02:30:00 +0300")),
                   FilterError::InvalidArgumentCount("expected at least 1, 0 given".to_owned()));
    }

    #[test]
    fn unit_date_extra_param() {
        assert_eq!(failed!(date,
                           tos!("13 Jun 2016 02:30:00 +0300"),
                           &[Value::scalar(0f32), Value::scalar(1f32)]),
                   FilterError::InvalidArgumentCount("expected at most 1, 2 given".to_owned()));
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_same_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Value::scalar(3i32)];
        let desired_result = tos!("2016-06-13 15:00:00 +0300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_previous_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Value::scalar(-13i32)];
        let desired_result = tos!("2016-06-12 23:00:00 -1300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_next_day() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Value::scalar(13i32)];
        let desired_result = tos!("2016-06-14 01:00:00 +1300");
        assert_eq!(unit!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_input_not_a_string() {
        let input = &Value::scalar(0f32);
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Value::scalar(0i32)];
        let desired_result = FilterError::InvalidType("Invalid date format".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_input_not_a_date_string() {
        let input = &tos!("blah blah blah");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), Value::scalar(0i32)];
        let desired_result = FilterError::InvalidType("Invalid date format".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_offset_not_a_num() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"), tos!("Hello")];
        let desired_result = FilterError::InvalidArgument(1, "Whole number expected".to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_zero_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[];
        let desired_result = FilterError::InvalidArgumentCount("expected at least 2, 0 given"
                                                                   .to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_one_argument() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z")];
        let desired_result = FilterError::InvalidArgumentCount("expected at least 2, 1 given"
                                                                   .to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }

    #[test]
    #[cfg(feature = "extra-filters")]
    fn unit_date_in_tz_three_arguments() {
        let input = &tos!("13 Jun 2016 12:00:00 +0000");
        let args = &[tos!("%Y-%m-%d %H:%M:%S %z"),
                     Value::scalar(0f32),
                     Value::scalar(1f32)];
        let desired_result = FilterError::InvalidArgumentCount("expected at most 2, 3 given"
                                                                   .to_owned());
        assert_eq!(failed!(date_in_tz, input, args), desired_result);
    }
}
