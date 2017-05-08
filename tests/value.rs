extern crate liquid;
extern crate serde_yaml;

#[macro_use]
extern crate difference;

use std::f32;

#[test]
pub fn serialize_num() {
    let actual = liquid::Value::Num(1f32);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n1", "", 0);

    let actual = liquid::Value::Num(-100f32);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n-100", "", 0);

    let actual = liquid::Value::Num(3.14e10f32);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n31399999488", "", 0);

    let actual = liquid::Value::Num(f32::NAN);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\nNaN", "", 0);

    let actual = liquid::Value::Num(f32::INFINITY);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\ninf", "", 0);
}

#[test]
pub fn deserialize_num() {
    let actual: liquid::Value = serde_yaml::from_str("---\n1").unwrap();
    assert_eq!(actual, liquid::Value::Num(1f32));

    let actual: liquid::Value = serde_yaml::from_str("---\n-100").unwrap();
    assert_eq!(actual, liquid::Value::Num(-100f32));

    let actual: liquid::Value = serde_yaml::from_str("---\n31399999488").unwrap();
    assert_eq!(actual, liquid::Value::Num(3.14e10f32));

    // Skipping NaN since equality fails

    let actual: liquid::Value = serde_yaml::from_str("---\ninf").unwrap();
    assert_eq!(actual, liquid::Value::Num(f32::INFINITY));
}

#[test]
pub fn serialize_bool() {
    let actual = liquid::Value::Bool(true);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\ntrue", "", 0);

    let actual = liquid::Value::Bool(false);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\nfalse", "", 0);
}

#[test]
pub fn deserialize_bool() {
    let actual: liquid::Value = serde_yaml::from_str("---\ntrue").unwrap();
    assert_eq!(actual, liquid::Value::Bool(true));

    let actual: liquid::Value = serde_yaml::from_str("---\nfalse").unwrap();
    assert_eq!(actual, liquid::Value::Bool(false));
}

#[test]
pub fn serialize_str() {
    let actual = liquid::Value::str("Hello");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\nHello", "", 0);

    let actual = liquid::Value::str("10");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n\"10\"", "", 0);

    let actual = liquid::Value::str("false");
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n\"false\"", "", 0);
}

#[test]
pub fn deserialize_str() {
    let actual: liquid::Value = serde_yaml::from_str("---\nHello").unwrap();
    assert_eq!(actual, liquid::Value::str("Hello"));

    let actual: liquid::Value = serde_yaml::from_str("\"10\"\n").unwrap();
    assert_eq!(actual, liquid::Value::str("10"));

    let actual: liquid::Value = serde_yaml::from_str("---\n\"false\"").unwrap();
    assert_eq!(actual, liquid::Value::str("false"));
}

#[test]
pub fn serialize_array() {
    let actual = vec![liquid::Value::Num(1f32),
                      liquid::Value::Bool(true),
                      liquid::Value::str("true")];
    let actual = liquid::Value::Array(actual);
    let actual = serde_yaml::to_string(&actual).unwrap();
    assert_diff!(&actual, "---\n- 1\n- true\n- \"true\"", "", 0);
}

#[test]
pub fn deserialize_array() {
    let actual: liquid::Value = serde_yaml::from_str("---\n- 1\n- true\n- \"true\"").unwrap();
    let expected = vec![liquid::Value::Num(1f32),
                        liquid::Value::Bool(true),
                        liquid::Value::str("true")];
    let expected = liquid::Value::Array(expected);
    assert_eq!(actual, expected);
}

#[test]
pub fn serialize_object() {
    // Skipping due to HashMap ordering issues
}

#[test]
pub fn deserialize_object() {
    let actual: liquid::Value = serde_yaml::from_str("---\nNum: 1\nBool: true\nStr: \"true\"")
        .unwrap();
    let expected: liquid::Object = [("Num".to_owned(), liquid::Value::Num(1f32)),
                                    ("Bool".to_owned(), liquid::Value::Bool(true)),
                                    ("Str".to_owned(), liquid::Value::str("true"))]
            .iter()
            .cloned()
            .collect();
    let expected = liquid::Value::Object(expected);
    assert_eq!(actual, expected);
}
