extern crate liquid_value;
extern crate serde_yaml;

#[macro_use]
extern crate difference;

use std::f64;

#[test]
pub fn serialize_num() {
    let actual = liquid_value::Value::scalar(1f64);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n1.0", "", 0);

    let actual = liquid_value::Value::scalar(-100f64);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n-100.0", "", 0);

    let actual = liquid_value::Value::scalar(3.14e_10f64);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n31400000000.0", "", 0);

    let actual = liquid_value::Value::scalar(f64::NAN);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n.nan", "", 0);

    let actual = liquid_value::Value::scalar(f64::INFINITY);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n.inf", "", 0);
}

#[test]
pub fn deserialize_num() {
    let actual: liquid_value::Value = serde_yaml::from_str("---\n1").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(1f64));

    let actual: liquid_value::Value = serde_yaml::from_str("---\n-100").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(-100f64));

    let actual: liquid_value::Value = serde_yaml::from_str("---\n31399999488").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(31399999488.0f64));

    // Skipping NaN since equality fails

    let actual: liquid_value::Value = serde_yaml::from_str("---\ninf").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(f64::INFINITY));
}

#[test]
pub fn serialize_bool() {
    let actual = liquid_value::Value::scalar(true);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\ntrue", "", 0);

    let actual = liquid_value::Value::scalar(false);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\nfalse", "", 0);
}

#[test]
pub fn deserialize_bool() {
    let actual: liquid_value::Value = serde_yaml::from_str("---\ntrue").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(true));

    let actual: liquid_value::Value = serde_yaml::from_str("---\nfalse").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar(false));
}

#[test]
pub fn serialize_nil() {
    let actual = liquid_value::Value::Nil;
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n~", "", 0);
}

#[test]
pub fn deserialize_nil() {
    let actual: liquid_value::Value = serde_yaml::from_str("---\n~").unwrap();
    assert_eq!(actual, liquid_value::Value::Nil);

    let actual: liquid_value::Value = serde_yaml::from_str("---\n- ").unwrap();
    assert_eq!(
        actual,
        liquid_value::Value::Array(vec![liquid_value::Value::Nil])
    );
}

#[test]
pub fn serialize_str() {
    let actual = liquid_value::Value::scalar("Hello");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\nHello", "", 0);

    let actual = liquid_value::Value::scalar("10");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n\"10\"", "", 0);

    let actual = liquid_value::Value::scalar("false");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n\"false\"", "", 0);
}

#[test]
pub fn deserialize_str() {
    let actual: liquid_value::Value = serde_yaml::from_str("---\nHello").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar("Hello"));

    let actual: liquid_value::Value = serde_yaml::from_str("\"10\"\n").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar("10"));

    let actual: liquid_value::Value = serde_yaml::from_str("---\n\"false\"").unwrap();
    assert_eq!(actual, liquid_value::Value::scalar("false"));
}

#[test]
pub fn serialize_array() {
    let actual = vec![
        liquid_value::Value::scalar(1f64),
        liquid_value::Value::scalar(true),
        liquid_value::Value::scalar("true"),
    ];
    let actual = liquid_value::Value::Array(actual);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n- 1.0\n- true\n- \"true\"", "", 0);
}

#[test]
pub fn deserialize_array() {
    let actual: liquid_value::Value = serde_yaml::from_str("---\n- 1\n- true\n- \"true\"").unwrap();
    let expected = vec![
        liquid_value::Value::scalar(1f64),
        liquid_value::Value::scalar(true),
        liquid_value::Value::scalar("true"),
    ];
    let expected = liquid_value::Value::Array(expected);
    assert_eq!(actual, expected);
}

#[test]
pub fn serialize_object() {
    // Skipping due to HashMap ordering issues
}

#[test]
pub fn deserialize_object() {
    let actual: liquid_value::Value =
        serde_yaml::from_str("---\nNum: 1\nBool: true\nStr: \"true\"").unwrap();
    let expected: liquid_value::Object = [
        ("Num".into(), liquid_value::Value::scalar(1f64)),
        ("Bool".into(), liquid_value::Value::scalar(true)),
        ("Str".into(), liquid_value::Value::scalar("true")),
    ]
    .iter()
    .cloned()
    .collect();
    let expected = liquid_value::Value::Object(expected);
    assert_eq!(actual, expected);
}
