
use std::str::FromStr;

use value::Value;

pub fn size<'a>(input : &'a str, _args: &Vec<Value>) -> String{
    input.len().to_string()
}

pub fn upcase<'a>(input: &'a str, _args: &Vec<Value>) -> String {
    let str = String::from(input);
    str.to_uppercase()
}

pub fn minus<'a>(input: &'a str, args: &Vec<Value>) -> String {
    let num = match f32::from_str(input) {
        Ok(n) => n,
        Err(_) => return input.to_string()
    };
    match args.first() {
        Some(&Value::Num(x)) => (num - x).to_string(),
        _ => num.to_string()
    }
}

#[test]
fn unit_size(){
    assert_eq!(size("abc", &vec![]), "3");
    assert_eq!(size("this has 22 characters", &vec![]), "22");
}

#[test]
fn unit_upcase() {
    assert_eq!(upcase("abc", &vec![]), "ABC");
    assert_eq!(upcase("Hello World 21", &vec![]), "HELLO WORLD 21");
}

#[test]
fn unit_minus() {
    assert_eq!(minus("2", &vec![Value::Num(1f32)]), "1");
    assert_eq!(minus("21.5", &vec![Value::Num(1.25)]), "20.25");
    assert_eq!(minus("invalid", &vec![Value::Num(1.25)]), "invalid");
    assert_eq!(minus("25", &vec![]), "25");
}
